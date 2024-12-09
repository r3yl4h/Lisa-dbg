use std::ffi::CStr;
use crate::usage::USAGE_PROC_ADDR;
use std::io;
use winapi::um::libloaderapi::{FreeLibrary, GetProcAddress, LoadLibraryA};
use winapi::um::winnt::HANDLE;
use crate::process::get_module;
use crate::ut::fmt::{print_lg, LevelPrint};

pub fn handle_get_proc_addr(h_proc: HANDLE, args: &[&str]) {
    if args.len() < 3 {
        eprintln!("{}", USAGE_PROC_ADDR);
        return;
    }
    unsafe {
        let new_dll = format!("{}\0", args[1].replace("\"", ""));
        let mut hdll = LoadLibraryA(new_dll.as_ptr() as *const i8);
        if hdll.is_null() {
            let last_err = io::Error::last_os_error();
            if (last_err.raw_os_error() == Some(193) || last_err.raw_os_error() == Some(126)) && !h_proc.is_null() {
                let mut n_dll = new_dll;
                n_dll.pop();
                match get_module(h_proc) {
                    Ok(modules) => {
                        for module in modules {
                            if CStr::from_ptr(module.szModule.as_ptr()).to_string_lossy() == n_dll {
                                hdll = module.hModule;
                                break;
                            }
                        }
                        if hdll.is_null() {
                            print_lg(LevelPrint::ErrorO, format!("unknow dll : {}", n_dll));
                        }
                    },
                    Err(e) => {
                        print_lg(LevelPrint::ErrorO, e);
                        return;
                    }
                }
            }
            print_lg(LevelPrint::ErrorO, format!("failed to get module handle : {}", io::Error::last_os_error()));
            return;
        }
        let new_func = format!("{}\0", args[2]);
        let addr_func = GetProcAddress(hdll, new_func.as_ptr() as *const i8);
        if addr_func.is_null() {
            print_lg(LevelPrint::ErrorO, format!("failed to get addr of func: {}", io::Error::last_os_error()));
            return;
        }
        print_lg(LevelPrint::DebugO, format!("address of function : {:#x}", addr_func as u64));
        FreeLibrary(hdll);
    }
}