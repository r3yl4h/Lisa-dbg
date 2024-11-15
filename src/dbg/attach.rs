use crate::dbg::exec;
use crate::ALL_ELM;
use std::ffi::CStr;
use std::{io, ptr};
use winapi::um::debugapi::DebugActiveProcess;
use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
use winapi::um::processthreadsapi::OpenProcess;
use winapi::um::winbase::QueryFullProcessImageNameA;
use winapi::um::winnt::{LPSTR, PROCESS_ALL_ACCESS};
use crate::ut::fmt::{print_lg, LevelPrint};

pub unsafe fn attach_dbg(pid: u32) {
    let h_proc = OpenProcess(PROCESS_ALL_ACCESS, 0, pid);
    if h_proc.is_null() || h_proc == INVALID_HANDLE_VALUE {
        print_lg(LevelPrint::ErrorO, format!("Failed to open pid {pid} : {}", io::Error::last_os_error()));
        return;
    }

    let mut path_size = winapi::shared::minwindef::MAX_PATH as u32;
    let mut path_buf = vec![0u8; path_size as usize];

    if QueryFullProcessImageNameA(h_proc, 0, path_buf.as_mut_ptr() as LPSTR, ptr::addr_of_mut!(path_size)) == 0 {
        print_lg(LevelPrint::ErrorO, format!("Failed to query full process image name : {}", io::Error::last_os_error()));
        return;
    }

    let path_str = CStr::from_ptr(path_buf.as_ptr() as *const i8).to_string_lossy();
    print_lg(LevelPrint::DebugO, format!("Process path: {}", path_str));

    ALL_ELM.file = Some(path_str.to_string());

    if let Err(e) = crate::pefile::parse_header() {
        print_lg(LevelPrint::Error, format!("Error when parsing PE headers: {e}"));
        CloseHandle(h_proc);
        return;
    }

    if DebugActiveProcess(pid) == 0 {
        print_lg(LevelPrint::Error, format!("Failed to debug process: {}", io::Error::last_os_error()));
        CloseHandle(h_proc);
        return;
    }

    exec::debug_loop(h_proc);
    CloseHandle(h_proc);
}