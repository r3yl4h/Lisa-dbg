use crate::{dbg, ut};
use crate::dbg::dbg_cmd::usages::USAGE_DBG_T;
use crate::pefile::{NtHeaders, NT_HEADER};
use ntapi::ntpsapi::{NtQueryInformationThread, THREAD_BASIC_INFORMATION};
use std::{io, ptr};
use winapi::shared::minwindef::LPVOID;
use winapi::um::handleapi::CloseHandle;
use winapi::um::processthreadsapi::{GetThreadContext, OpenThread, SetThreadContext};
use winapi::um::winbase::{Wow64GetThreadContext, Wow64SetThreadContext};
use winapi::um::winnt::{
    CONTEXT, CONTEXT_ALL, HANDLE, THREAD_ALL_ACCESS, WOW64_CONTEXT, WOW64_CONTEXT_ALL,
};
use crate::ut::cast::str_to;
use crate::ut::fmt::*;


pub fn change_dbg_thread(linev: &[&str], ctx: *mut CONTEXT, h_proc: HANDLE, h_thread1: &mut HANDLE, addr_func: &mut u64) {
    if linev.len() < 2 {
        print_lg(LevelPrint::WarningO, USAGE_DBG_T);
        return;
    }

    let tip = match str_to::<u32>(linev[1]) {
        Ok(tid) => tid,
        Err(e) => {
            print_lg(LevelPrint::Error, format!("failed to parse thread id: {e}"));
            return;
        }
    };

    unsafe {
        let h_thread = OpenThread(THREAD_ALL_ACCESS, 0, tip);
        if h_thread.is_null() {
            print_lg(LevelPrint::Error, format!("failed to open thread: {}", io::Error::last_os_error()));
            return;
        }

        match NT_HEADER {
            Some(NtHeaders::Headers64(_)) => {
                if SetThreadContext(*h_thread1, ctx) == 0 {
                    print_lg(LevelPrint::Error, format!("failed to set thread context: {}", io::Error::last_os_error()));
                    return;
                }
                CloseHandle(*h_thread1);
                *h_thread1 = h_thread;
                match ut::mem::alloc_size_align::<CONTEXT>() {
                    Ok(pctx) => {
                        (*pctx).ContextFlags = CONTEXT_ALL;
                        if GetThreadContext(h_thread, pctx) == 0 {
                            print_lg(LevelPrint::Error, format!("failed to get thread context: {}", io::Error::last_os_error()));
                            return;
                        }
                        *ctx = *pctx;
                        dbg::dbg_cmd::init_cm(*ctx, h_proc, h_thread, addr_func);
                    }
                    Err(e) => {
                        print_lg(LevelPrint::Error, e);
                        return;
                    }
                }
            }
            Some(NtHeaders::Headers32(_)) => {
                if Wow64SetThreadContext(*h_thread1, ctx as *const WOW64_CONTEXT) == 0 {
                    print_lg(LevelPrint::Error, format!("failed to set thread context: {}", io::Error::last_os_error()));
                    return;
                }
                CloseHandle(*h_thread1);
                *h_thread1 = h_thread;

                match ut::mem::alloc_size_align::<WOW64_CONTEXT>() {
                    Ok(pctx) => {
                        (*pctx).ContextFlags = WOW64_CONTEXT_ALL;
                        if Wow64GetThreadContext(h_thread, pctx) == 0 {
                            print_lg(LevelPrint::Error, format!("failed to get thread context: {}", io::Error::last_os_error()));
                            return;
                        }
                        let w64_ctx = ctx as *mut WOW64_CONTEXT;
                        *w64_ctx = *pctx;

                        let mut addr2 = *addr_func as u32;
                        dbg::dbg_cmd::x32::init_cm(*w64_ctx, h_proc, h_thread, &mut addr2);
                        *addr_func = addr2 as u64;
                    }
                    Err(e) => print_lg(LevelPrint::Error, e)
                }
            }
            _ => {}
        }
        print_lg(LevelPrint::DebugO, format!("now you are on the thread {tip}"));
    }
}


pub fn get_thread_now(h_thread: HANDLE) {
    unsafe {
        let tbi: THREAD_BASIC_INFORMATION = std::mem::zeroed();
        let ntstatus = NtQueryInformationThread(h_thread, ntapi::ntpsapi::ThreadBasicInformation, ptr::addr_of!(tbi) as LPVOID, size_of::<THREAD_BASIC_INFORMATION>() as u32, &mut 0);
        if ntstatus == 0 {
            println!("{}Thread: ", GREEN_COL);
            println!("    {}Thread id : {}", MAGENTA, tbi.ClientId.UniqueThread as u32);
            println!("    {}Owner pid : {}", VALUE_COLOR, tbi.ClientId.UniqueProcess as u32);
            println!();
        } else {
            print_lg(LevelPrint::ErrorO, format!("failed to query info of current thread to debug with ntstatus : {ntstatus}"));
        }
    }
}
