use std::alloc::{alloc, Layout};
use std::{io, ptr};
use anyhow::anyhow;
use winapi::shared::minwindef::LPVOID;
use winapi::shared::ntdef::HANDLE;
use winapi::um::memoryapi::{ReadProcessMemory, VirtualProtectEx};
use winapi::um::winnt::PAGE_EXECUTE_READWRITE;
use crate::pefile::get_section_of_rva;
use crate::ut::fmt::{print_lg, LevelPrint};

pub fn alloc_size_align<T: Sized>() -> Result<*mut T, anyhow::Error>{
    let layout = Layout::from_size_align(size_of::<T>(), align_of::<T>())?;
    Ok(unsafe {alloc(layout) as *mut T})
}






pub fn read_mem(h_proc: HANDLE, addr: u64, buffer: &mut [u8]) -> Result<(), anyhow::Error> {
    let mut old_protect: u32 = 0;
    if h_proc.is_null() {
        return if let Some(sec) = get_section_of_rva(addr) {
            let offset = addr as usize - sec.addr as usize;
            let tmp = &sec.content[offset..];
            buffer.copy_from_slice(&tmp[..buffer.len()]);
            Ok(())
        } else {
            Err(anyhow!("invalid address"))
        }
    }
    unsafe {
        if VirtualProtectEx(h_proc, addr as LPVOID, buffer.len(), PAGE_EXECUTE_READWRITE, &mut old_protect) == 0 {
            return Err(anyhow!("Failed to remove memory protection at address {:#x}: {}", addr, io::Error::last_os_error()));
        }
        if ReadProcessMemory(h_proc, addr as LPVOID, buffer.as_mut_ptr() as LPVOID, buffer.len(), ptr::null_mut()) == 0 {
            print_lg(LevelPrint::ErrorO, format!("Failed to read process memory: {}", io::Error::last_os_error()));
            return Err(anyhow!("Failed to read process memory: {}", io::Error::last_os_error()));
        }
        if VirtualProtectEx(h_proc, addr as LPVOID, buffer.len(), old_protect, &mut old_protect) == 0 {
            return Err(anyhow!("Failed to restore memory protection at address {:#x}: {}", addr, io::Error::last_os_error()));
        }
    }
    Ok(())
}