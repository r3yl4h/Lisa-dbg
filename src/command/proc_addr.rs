use crate::usage::USAGE_PROC_ADDR;
use std::io;
use winapi::um::libloaderapi::{FreeLibrary, GetProcAddress, LoadLibraryA};
use crate::ut::fmt::{print_lg, LevelPrint};

pub fn handle_get_proc_addr(args: &[&str]) {
    if args.len() < 3 {
        eprintln!("{}", USAGE_PROC_ADDR);
        return;
    }
    unsafe {
        let new_dll = format!("{}\0", args[1].replace("\"", ""));
        let hdll = LoadLibraryA(new_dll.as_ptr() as *const i8);
        if hdll.is_null() {
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
