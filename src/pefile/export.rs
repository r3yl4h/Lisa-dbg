use std::{io, mem, ptr};
use std::ffi::CStr;
use anyhow::anyhow;
use winapi::shared::minwindef::LPVOID;
use winapi::um::memoryapi::ReadProcessMemory;
use winapi::um::winnt::{HANDLE, IMAGE_DOS_HEADER, IMAGE_EXPORT_DIRECTORY, IMAGE_NT_HEADERS32, IMAGE_NT_HEADERS64};
use crate::symbol::{SrcFile, SymbolFile};

pub fn get_export_func_in_dll(h_proc: HANDLE, base: u64) -> Result<Vec<SymbolFile>, anyhow::Error> {
    let mut result = Vec::new();
    let mut addr = base;
    unsafe {
        let mut dos_header: IMAGE_DOS_HEADER = mem::zeroed();
        if ReadProcessMemory(h_proc, addr as LPVOID, ptr::addr_of_mut!(dos_header) as LPVOID, size_of::<IMAGE_DOS_HEADER>(), &mut 0) == 0 {
            return Err(anyhow!("failed to read dos headers : {}", io::Error::last_os_error()))
        }
        addr += (dos_header.e_lfanew + 4) as u64;

        let mut machine = 0u16;
        if ReadProcessMemory(h_proc, addr as LPVOID, ptr::addr_of_mut!(machine) as LPVOID, 2, &mut 0) == 0 {
            return Err(anyhow!("failed to read machine header : {}", io::Error::last_os_error()))
        }

        addr -= 4;
        let export_dir;
        match machine {
            0x014c => {
                let mut nt32: IMAGE_NT_HEADERS32 = mem::zeroed();
                if ReadProcessMemory(h_proc, addr as LPVOID, ptr::addr_of_mut!(nt32) as LPVOID, mem::size_of::<IMAGE_NT_HEADERS32>(), &mut 0) == 0 {
                    return Err(anyhow!("failed to read nt headers : {}", io::Error::last_os_error()))
                };
                export_dir = nt32.OptionalHeader.DataDirectory[0];
            }
            0x8664 => {
                let mut nt64: IMAGE_NT_HEADERS64 = mem::zeroed();
                if ReadProcessMemory(h_proc, addr as LPVOID, ptr::addr_of_mut!(nt64) as LPVOID, mem::size_of::<IMAGE_NT_HEADERS64>(), &mut 0) == 0 {
                    return Err(anyhow!("failed to read nt headers : {}", io::Error::last_os_error()))
                }
                export_dir = nt64.OptionalHeader.DataDirectory[0];
            }
            _ => return Err(anyhow!("invalid machine header: {:#x}", machine))
        }

        let mut img_export: IMAGE_EXPORT_DIRECTORY = mem::zeroed();
        addr = export_dir.VirtualAddress as u64 + base;

        if ReadProcessMemory(h_proc, addr as LPVOID, ptr::addr_of_mut!(img_export) as LPVOID, mem::size_of::<IMAGE_EXPORT_DIRECTORY>(), &mut 0) == 0 {
            return Err(anyhow!("failed to read export directory : {}", io::Error::last_os_error()))
        }
        let mut addr_func = vec![0u32;img_export.NumberOfFunctions as usize];
        let mut addr_name = vec![0u32;img_export.NumberOfFunctions as usize];
        
        addr = img_export.AddressOfFunctions as u64 + base;
        if ReadProcessMemory(h_proc, addr as LPVOID, addr_func.as_mut_ptr() as LPVOID, addr_func.len() * 4, &mut 0) == 0 {
            return Err(anyhow!("failed to read address function : {}", io::Error::last_os_error()))
        }
        
        addr = img_export.AddressOfNames as u64 + base;
        if ReadProcessMemory(h_proc, addr as LPVOID, addr_name.as_mut_ptr() as LPVOID, addr_name.len() * 4, &mut 0) == 0 {
            return Err(anyhow!("failed to read address name : {}", io::Error::last_os_error()))
        }
        
        for i in 0..img_export.NumberOfNames as usize {
            addr = addr_name[i] as u64 + base;
            let mut name_buf = [0u8;260];
            if ReadProcessMemory(h_proc, addr as LPVOID, name_buf.as_mut_ptr() as LPVOID, name_buf.len(), &mut 0) == 0 {
                return Err(anyhow!("failed to read name : {}", io::Error::last_os_error()))
            }
            let mut sym = SymbolFile::default();
            sym.name = CStr::from_ptr(name_buf.as_ptr() as *const i8).to_string_lossy().to_string();
            sym.offset = addr_func[i] as i64;
            sym.src_file = SrcFile::Dll(base);
            result.push(sym);
        }
    }
    Ok(result)
}
