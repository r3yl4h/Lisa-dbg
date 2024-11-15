use crate::dbg::BASE_ADDR;
use crate::symbol;
use std::{io, ptr};
use winapi::shared::minwindef::LPVOID;
use winapi::shared::ntdef::HANDLE;
use winapi::um::memoryapi::{ReadProcessMemory, VirtualProtectEx, WriteProcessMemory};
use winapi::um::winnt::PAGE_EXECUTE_READWRITE;
use crate::ut::fmt::{print_lg, LevelPrint};

pub mod breakpoint;
pub mod deref_mem;
pub mod finder;
pub mod func;
pub mod mem_info;
pub mod set;
pub mod stack;
pub mod watchpoint;

pub unsafe fn set_addr_over(h_proc: HANDLE, over_func: u64, save_insn: &mut u8) -> Result<(), String> {
    let mut old_protect = 0;
    if VirtualProtectEx(h_proc, over_func as LPVOID, 1, PAGE_EXECUTE_READWRITE, &mut old_protect) == 0 {
        return Err(format!("an error occurred while removing memory protection at address {:#x}", over_func));
    }
    if ReadProcessMemory(h_proc, over_func as LPVOID, ptr::addr_of_mut!(*save_insn) as LPVOID, 1, &mut 0) == 0 {
        return Err(format!("failed to read memory at address {:#x} : {}", over_func, io::Error::last_os_error()));
    }
    if WriteProcessMemory(h_proc, over_func as LPVOID, &0xc3u8 as *const u8 as LPVOID, 1, &mut 0) == 0 {
        return Err(format!("an error occurred while writing to memory at address {:#x}", over_func));
    } else {
        print_lg(LevelPrint::Debug, format!("now, function {} will no longer run", func_format(over_func)));
    }
    if VirtualProtectEx(h_proc, over_func as LPVOID, 1, old_protect, &mut old_protect) == 0 {
        return Err(format!("an error occurred while restauring memory protection at address {:#x}", over_func));
    }
    Ok(())
}

unsafe fn func_format(addr: u64) -> String {
    if let Some(sym) = (*&raw mut symbol::SYMBOLS_V).symbol_file.iter().find(|s| s.offset + BASE_ADDR as i64 == addr as i64) {
        format!("{} at address {:#x}", sym.name, addr)
    } else {
        format!("at address {:#x}", addr)
    }
}
