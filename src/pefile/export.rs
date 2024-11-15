use std::{io, mem, ptr};
use std::ffi::CStr;
use anyhow::anyhow;
use winapi::shared::minwindef::LPVOID;
use winapi::um::memoryapi::ReadProcessMemory;
use winapi::um::winnt::{HANDLE, IMAGE_DOS_HEADER, IMAGE_EXPORT_DIRECTORY, IMAGE_NT_HEADERS32, IMAGE_NT_HEADERS64, IMAGE_SECTION_HEADER};
use winapi::um::winnt::{IMAGE_FILE_MACHINE_AMD64, IMAGE_FILE_MACHINE_I386, IMAGE_FILE_MACHINE_IA64};
use crate::symbol::{SrcFile, SymbolFile};

pub fn get_export_func_in_dll(h_proc: HANDLE, base: u64) -> Result<Vec<SymbolFile>, anyhow::Error> {
    let mut addr = base;
    let mut result = Vec::new();
    unsafe {
        let mut dos_header = mem::zeroed::<IMAGE_DOS_HEADER>();
        if ReadProcessMemory(h_proc, addr as LPVOID, ptr::addr_of_mut!(dos_header) as LPVOID, size_of::<IMAGE_DOS_HEADER>(), &mut 0) == 0 {
            return Err(anyhow!("failed to read memory for get dos header : {}", io::Error::last_os_error()));
        }
        if dos_header.e_magic != 0x5a4d {
            return Err(anyhow!("the magic number of the dll is corrupted"));
        }
        let mut machine = 0u16;
        addr += dos_header.e_lfanew as u64 + 4;
        if ReadProcessMemory(h_proc, addr as LPVOID, ptr::addr_of_mut!(machine) as LPVOID, 2, &mut 0) == 0 {
            return Err(anyhow!("failed to read process memory for get machine value: {}", io::Error::last_os_error()));
        }
        addr -= 4;
        let mut r_byte = 0;
        let (export_r, num_sec) = match machine {
            IMAGE_FILE_MACHINE_I386 => {
                let mut nt32 = mem::zeroed::<IMAGE_NT_HEADERS32>();
                if ReadProcessMemory(h_proc, addr as LPVOID, ptr::addr_of_mut!(nt32) as LPVOID, size_of::<IMAGE_NT_HEADERS32>(), &mut r_byte) == 0 {
                    return Err(anyhow!("failed to read NT_HEADERS32 : {}", io::Error::last_os_error()));
                }
                addr += r_byte as u64;
                (nt32.OptionalHeader.DataDirectory[0], nt32.FileHeader.NumberOfSections)
            }
            IMAGE_FILE_MACHINE_AMD64 | IMAGE_FILE_MACHINE_IA64 => {
                let mut nt64 = mem::zeroed::<IMAGE_NT_HEADERS64>();
                if ReadProcessMemory(h_proc, addr as LPVOID, ptr::addr_of_mut!(nt64) as LPVOID, size_of::<IMAGE_NT_HEADERS64>(), &mut r_byte) == 0 {
                    return Err(anyhow!("failed to read NT_HEADERS64 : {}", io::Error::last_os_error()));
                }
                addr += r_byte as u64;
                (nt64.OptionalHeader.DataDirectory[0], nt64.FileHeader.NumberOfSections)
            }
            _ => return Err(anyhow!("unsupported machine number")),
        };
        let mut sectionv = Vec::with_capacity(num_sec as usize);
        for _ in 0..num_sec {
            r_byte = 0;
            let mut section = mem::zeroed::<IMAGE_SECTION_HEADER>();
            if ReadProcessMemory(h_proc, addr as LPVOID, ptr::addr_of_mut!(section) as LPVOID, size_of::<IMAGE_SECTION_HEADER>(), &mut r_byte) == 0 {
                return Err(anyhow!("failed to read section header at address {:#x}: {}", addr, io::Error::last_os_error()));
            }
            addr += r_byte as u64;
            sectionv.push(section);
        }
        addr = base + export_r.VirtualAddress as u64;
        let mut export = mem::zeroed::<IMAGE_EXPORT_DIRECTORY>();
        if ReadProcessMemory(h_proc, addr as LPVOID, ptr::addr_of_mut!(export) as LPVOID, size_of::<IMAGE_EXPORT_DIRECTORY>(), &mut r_byte) == 0 {
            return Err(anyhow!("failed to read export directory : {}", io::Error::last_os_error()));
        }
        let mut addr_name: Vec<u32> = vec![0u32; export.NumberOfNames as usize];
        let mut addr_func: Vec<u32> = vec![0u32; export.NumberOfFunctions as usize];

        addr = base + export.AddressOfNames as u64;
        if ReadProcessMemory(h_proc, addr as LPVOID, addr_name.as_mut_ptr() as LPVOID, 4 * export.NumberOfFunctions as usize, &mut 0) == 0 {
            return Err(anyhow!("failed to read function address : {}", io::Error::last_os_error()));
        }

        addr = base + export.AddressOfFunctions as u64;
        if ReadProcessMemory(h_proc, addr as LPVOID, addr_func.as_mut_ptr() as LPVOID, 4 * export.NumberOfNames as usize, &mut 0) == 0 {
            return Err(anyhow!("failed to read function name : {}", io::Error::last_os_error()));
        }
        for i in 0..export.NumberOfNames as usize {
            let mut sym = SymbolFile::default();
            addr = addr_name[i] as u64 + base;
            let mut name = [0u8;260];
            if ReadProcessMemory(h_proc, addr as LPVOID, name.as_mut_ptr() as LPVOID, 260, &mut 0) == 0 {
                return Err(anyhow!("failed to read name : {}", io::Error::last_os_error()));
            }
            sym.name = CStr::from_ptr(name.as_ptr() as *const i8).to_string_lossy().to_string();
            sym.offset = addr_func[i] as i64;
            sym.src_file = SrcFile::Dll(base);
            result.push(sym);
        }
        Ok(result)
    }
}
