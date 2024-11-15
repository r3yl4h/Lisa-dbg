use crate::dbg::dbg_cmd::x32::info_reg::ToValue32;
use crate::dbg::dbg_cmd::x64::info_reg::{ToValue, Value};
use crate::dbg::RealAddr;
use crate::symbol::SYMBOLS_V;
use crate::usage;
use std::{io, ptr};
use winapi::shared::minwindef::LPVOID;
use winapi::shared::ntdef::HANDLE;
use winapi::um::memoryapi::{VirtualProtectEx, VirtualQueryEx};
use winapi::um::winnt::{
    CONTEXT, MEMORY_BASIC_INFORMATION, PAGE_EXECUTE, PAGE_EXECUTE_READ, PAGE_EXECUTE_READWRITE,
    PAGE_EXECUTE_WRITECOPY, PAGE_NOACCESS, PAGE_READONLY, PAGE_READWRITE, PAGE_WRITECOPY,
    WOW64_CONTEXT,
};
use crate::pefile::{NtHeaders, NT_HEADER};
use crate::ut::cast::str_to;
use crate::ut::fmt::*;

fn protect_to_str(protect: u32) -> &'static str {
    match protect {
        PAGE_NOACCESS => "PAGE_NOACCESS",
        PAGE_READONLY => "PAGE_READONLY",
        PAGE_READWRITE => "PAGE_READWRITE",
        PAGE_WRITECOPY => "PAGE_WRITECOPY",
        PAGE_EXECUTE => "PAGE_EXECUTE",
        PAGE_EXECUTE_READ => "PAGE_EXECUTE_READ",
        PAGE_EXECUTE_READWRITE => "PAGE_EXECUTE_READWRITE",
        PAGE_EXECUTE_WRITECOPY => "PAGE_EXECUTE_WRITECOPY",
        _ => "UNKNOWN",
    }
}

fn get_new_protect(arg_str: &str) -> u32 {
    match arg_str {
        "noaccess" => PAGE_NOACCESS,
        "readonly" | "r" => PAGE_READONLY,
        "readwrite" | "rw" | "wr" => PAGE_READWRITE,
        "writecopy" | "w" => PAGE_WRITECOPY,
        "execute" | "x" => PAGE_EXECUTE,
        "exec_read" | "xr" | "rx" => PAGE_EXECUTE_READ,
        "exec_readwrite" | "rwx" | "wrx" | "xwr" => PAGE_EXECUTE_READWRITE,
        "exec_writecopy" | "xw" => PAGE_EXECUTE_WRITECOPY,
        _ => 0,
    }
}

fn get_size(arg: &[&str], h_proc: HANDLE, addr: u64) -> Result<usize, String> {
    if arg.len() == 3 {
        match str_to::<usize>(arg[2]) {
            Ok(size) => Ok(size),
            Err(e) => Err(format!("Invalid size : {e}")),
        }
    } else {
        unsafe {
            let mut mem_info: MEMORY_BASIC_INFORMATION = std::mem::zeroed();
            if VirtualQueryEx(h_proc, addr as LPVOID, &mut mem_info, size_of::<MEMORY_BASIC_INFORMATION>()) == 0 {
                return Err(format!("failed to query memory info of {:#x} : {}", addr, io::Error::last_os_error()));
            }
            Ok(mem_info.RegionSize as usize)
        }
    }
}

fn change_protect_mem(h_proc: HANDLE, addr: u64, size_mem: usize, new_protect: u32) {
    unsafe {
        let mut old_protect = 0;
        if VirtualProtectEx(h_proc, addr as LPVOID, size_mem, new_protect, &mut old_protect) == 0 {
            print_lg(LevelPrint::Error, format!("Failed to change memory protection : {}", io::Error::last_os_error()));
            return;
        }
        print_lg(LevelPrint::DebugO, format!("Memory protection changed successfully, old protect: {} ({})", old_protect, protect_to_str(old_protect)));
    }
}



pub fn change_protect(h_proc: HANDLE, ctx: *const CONTEXT, arg: &[&str]) {
    if arg.len() < 2 {
        print_lg(LevelPrint::WarningO, usage::USAGE_SET_PROTECT);
        return;
    }

    let target = arg[0];
    let addr = match str_to::<u64>(target) {
        Ok(addr) => addr,
        Err(_) => unsafe {
            if let Some(sym) = (*ptr::addr_of!(SYMBOLS_V)).symbol_file.iter().find(|s| s.name == target) {
                sym.real_addr(ctx)
            } else {
                match NT_HEADER.unwrap() {
                    NtHeaders::Headers64(_) => {
                        let v = (*ctx).str_to_value_ctx(target);
                        match v {
                            Value::U64(addr) => addr,
                            Value::U128(_) => {
                                print_lg(LevelPrint::Error, "you can't specify a SIMD register".to_string());
                                return;
                            }
                            _ => {
                                print_lg(LevelPrint::Error, format!("Unknown target: {target}"));
                                return;
                            }
                        }
                    }
                    NtHeaders::Headers32(_) => {
                        let v = (*(ctx as *const WOW64_CONTEXT)).str_to_ctx(target);
                        if v == 0 {
                            print_lg(LevelPrint::Error, format!("Unknown target: {target}"));
                            return;
                        }
                        v as u64
                    }
                }
            }
        }
    };

    let new_protect = get_new_protect(arg[1]);
    if new_protect == 0 {
        print_lg(LevelPrint::Error, format!("Invalid protection flag: {}", arg[1]));
        return;
    }
    let size_mem = match get_size(arg, h_proc, addr) {
        Ok(size) => size,
        Err(e) => {
            print_lg(LevelPrint::Error, e);
            return;
        }
    };
    change_protect_mem(h_proc, addr, size_mem, new_protect);
}
