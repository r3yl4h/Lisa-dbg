use crate::cli::{AfterB, ALL_ELM};
use crate::command::hook::Hook;
use crate::dbg::{memory, DbgState, RealAddr, BASE_ADDR};
use crate::pefile::{NtHeaders, NT_HEADER};
use std::{io, ptr};
use winapi::shared::minwindef::{FALSE, LPVOID};
use winapi::um::memoryapi::{ReadProcessMemory, WriteProcessMemory};
use winapi::um::minwinbase::DEBUG_EVENT;
use winapi::um::processthreadsapi::{GetThreadContext, OpenThread, SetThreadContext};
use winapi::um::winbase::{Wow64GetThreadContext, Wow64SetThreadContext};
use winapi::um::winnt::{CONTEXT, CONTEXT_ALL, HANDLE, THREAD_ALL_ACCESS, THREAD_GET_CONTEXT, THREAD_SET_CONTEXT, WOW64_CONTEXT, WOW64_CONTEXT_ALL};
use crate::ut;
use crate::ut::fmt::{print_lg, LevelPrint};

pub fn handle_single_step(debug_event: DEBUG_EVENT, b_addr: u64, h_proc: HANDLE, c_dbg: &mut DbgState) {
    unsafe {
        let mut h_thread = OpenThread(THREAD_ALL_ACCESS, 0, debug_event.dwThreadId);
        if h_thread.is_null() {
            print_lg(LevelPrint::Error, format!("failed to open thread {} : {}", debug_event.dwThreadId, io::Error::last_os_error()));
            return;
        }
        match NT_HEADER.unwrap() {
            NtHeaders::Headers32(_) => {
                match ut::mem::alloc_size_align::<WOW64_CONTEXT>() {
                    Ok(pctx) => {
                        let ctx = &mut *pctx;
                        ctx.ContextFlags = WOW64_CONTEXT_ALL;
                        if Wow64GetThreadContext(h_thread, ctx) == 0 {
                            print_lg(LevelPrint::Error, format!("failed to get thread context of thread {} : {}", debug_event.dwThreadId, io::Error::last_os_error()));
                            return;
                        }
                        if (*&raw const ALL_ELM).watchpts.iter().any(|w| w.real_addr32(*ctx) == b_addr as u32) {
                            memory::watchpoint::handle_watchpoint32(debug_event, h_proc, &mut h_thread, ctx, c_dbg);
                        }
                    }
                    Err(e) => print_lg(LevelPrint::Error, format!("failed to get thread context: {}", e)),
                }
            }
            NtHeaders::Headers64(_) => {
                match ut::mem::alloc_size_align::<CONTEXT>() {
                    Ok(pctx) => {
                        let mut ctx = &mut *pctx;
                        ctx.ContextFlags = CONTEXT_ALL;
                        if GetThreadContext(h_thread, ctx) == 0 {
                            print_lg(LevelPrint::Error, format!("failed to get thread context of thread {} : {}", debug_event.dwThreadId, io::Error::last_os_error()));
                            return;
                        }
                        if (*&raw const ALL_ELM).watchpts.iter().any(|w| w.real_addr64(*ctx) == b_addr) {
                            memory::watchpoint::handle_watchpoint64(debug_event, h_proc, &mut h_thread, &mut ctx, c_dbg);
                        }
                        if SetThreadContext(h_thread, ctx) == 0 {
                            print_lg(LevelPrint::Error, format!("failed to set thread context of thread {} : {}", debug_event.dwThreadId, io::Error::last_os_error()));
                        }
                    }
                    Err(e) => print_lg(LevelPrint::Error, format!("failed to get thread context: {} : {}", debug_event.dwThreadId, e)),
                }
            }
        }
    }
}



pub unsafe fn handle_after_b(h_proc: HANDLE, ab: AfterB, c_dbg: &mut DbgState, debug_event: DEBUG_EVENT) {
    if WriteProcessMemory(h_proc, ab.after_b as LPVOID, ptr::addr_of!(ab.last_oc) as LPVOID, 1, &mut 0) == 0 {
        print_lg(LevelPrint::Critical1(debug_event.dwProcessId, c_dbg), format!("failed to restore the original byte at address {:#x} : {}", ab.after_b, io::Error::last_os_error()));
        return;
    }
    let h_thread = OpenThread(THREAD_ALL_ACCESS, 0, debug_event.dwThreadId);
    if h_thread.is_null() {
        print_lg(LevelPrint::Critical1(debug_event.dwProcessId, c_dbg), format!("Failed to open thread {}: {}", debug_event.dwThreadId, io::Error::last_os_error()));
        return;
    }
    match NT_HEADER.unwrap() {
        NtHeaders::Headers32(_) => {
            match ut::mem::alloc_size_align::<WOW64_CONTEXT>() {
                Ok(pctx) => {
                    let ctx = &mut *pctx;
                    ctx.ContextFlags = WOW64_CONTEXT_ALL;
                    if Wow64GetThreadContext(h_thread, ctx) == 0 {
                        print_lg(LevelPrint::Error, format!("failed to get thread context of thread {} : {}", debug_event.dwThreadId, io::Error::last_os_error()));
                        return;
                    }
                    ctx.Eip -= 1;
                    if Wow64SetThreadContext(h_thread, ctx) == 0 {
                        print_lg(LevelPrint::Critical1(debug_event.dwProcessId, c_dbg), format!("Failed to set thread context: {}", io::Error::last_os_error()));
                        return;
                    }
                }
                Err(e) => print_lg(LevelPrint::Error, format!("failed to get thread context: {} : {}", debug_event.dwThreadId, e)),
            }
        }
        NtHeaders::Headers64(_) => {
            match ut::mem::alloc_size_align::<CONTEXT>() {
                Ok(pctx) => {
                    let ctx = &mut *pctx;
                    ctx.ContextFlags = CONTEXT_ALL;
                    if GetThreadContext(h_thread, ctx) == 0 {
                        print_lg(LevelPrint::Critical1(debug_event.dwProcessId, c_dbg), format!("Failed to get thread context: {}", io::Error::last_os_error()));
                        return;
                    }
                    ctx.Rip -= 1;
                    if SetThreadContext(h_thread, ctx) == 0 {
                        print_lg(LevelPrint::Critical1(debug_event.dwProcessId, c_dbg), format!("Failed to set thread context: {}", io::Error::last_os_error()));
                        return;
                    }
                }
                Err(e) => print_lg(LevelPrint::Error, format!("failed to get thread context: {} : {}", debug_event.dwThreadId, e)),
            }
        }
    }
    let mut last_oc = 0u8;
    if ReadProcessMemory(h_proc, ab.last_addr_b as LPVOID, ptr::addr_of_mut!(last_oc) as LPVOID, 1, &mut 0) == 0 {
        print_lg(LevelPrint::Critical1(debug_event.dwProcessId, c_dbg), format!("failed to read memory at address {:#x} : {}", ab.last_addr_b, io::Error::last_os_error()));
        return;
    }
    if WriteProcessMemory(h_proc, ab.last_addr_b as LPVOID, &0xccu8 as *const u8 as LPVOID, 1, &mut 0) == 0 {
        print_lg(LevelPrint::Error, format!("Failed to rewrite breakpoint at address {:#x} : {}", ab.last_addr_b, io::Error::last_os_error()));
        return;
    }
}



pub unsafe fn handle_hook_func(_h_proc: HANDLE, func_hook: Hook, debug_event: DEBUG_EVENT, c_dbg: &mut DbgState) {
    let h_thread = OpenThread(THREAD_GET_CONTEXT | THREAD_SET_CONTEXT, FALSE, debug_event.dwThreadId);
    if !h_thread.is_null() {
        match NT_HEADER {
            Some(NtHeaders::Headers64(_)) => {
                match ut::mem::alloc_size_align::<CONTEXT>() {
                    Ok(pctx) => {
                        let ctx = &mut *pctx;
                        ctx.ContextFlags = CONTEXT_ALL;
                        if GetThreadContext(h_thread, ctx) == 0 {
                            print_lg(LevelPrint::Critical1(debug_event.dwProcessId, c_dbg), format!("failed to get thread context : {}", io::Error::last_os_error()));
                            return;
                        }
                        let addr_target = func_hook.replacen + BASE_ADDR;
                        ctx.Rip = addr_target;
                        if SetThreadContext(h_thread, ctx) == 0 {
                            print_lg(LevelPrint::Critical1(debug_event.dwProcessId, c_dbg), format!("error when setting thread context : {}", io::Error::last_os_error()));
                        } else {
                            print_lg(LevelPrint::Debug, format!("the program execution flow has been redirected to the address {:#x}", addr_target));
                        }
                    }
                    Err(e) => print_lg(LevelPrint::Error, format!("failed to get thread context: {} : {}", debug_event.dwProcessId, e)),
                }
            }
            Some(NtHeaders::Headers32(_)) => {
                match ut::mem::alloc_size_align::<WOW64_CONTEXT>() {
                    Ok(pctx) => {
                        let ctx = &mut *pctx;
                        ctx.ContextFlags = WOW64_CONTEXT_ALL;
                        if Wow64GetThreadContext(h_thread, ctx) == 0 {
                            print_lg(LevelPrint::Critical1(debug_event.dwProcessId, c_dbg), format!("failed to get thread context : {}", io::Error::last_os_error()));
                        } else {
                            let addr_target = func_hook.replacen + BASE_ADDR;
                            ctx.Eip = addr_target as u32;
                            if Wow64SetThreadContext(h_thread, ctx) == 0 {
                                print_lg(LevelPrint::Critical1(debug_event.dwProcessId, c_dbg), format!("error when setting thread context : {}", io::Error::last_os_error()));
                            } else {
                                print_lg(LevelPrint::Debug, format!("the program execution flow has been redirected to the address {:#x}", addr_target));
                            }
                        }
                    }
                    Err(e) => print_lg(LevelPrint::Error, format!("failed to get thread context: {} : {}", debug_event.dwProcessId, e)),
                }
            }
            None => {}
        }
    } else {
        print_lg(LevelPrint::Critical1(debug_event.dwProcessId, c_dbg), format!("Failed to open thread: {}", io::Error::last_os_error()));
    }
}
