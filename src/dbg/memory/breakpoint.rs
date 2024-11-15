use crate::cli::AfterB;
use crate::dbg::memory::stack::{get_frame_st, get_frame_st32, get_real_frame, ST_FRAME};
use crate::dbg::{dbg_cmd, DbgState, BASE_ADDR};
use crate::pefile::{NtHeaders, NT_HEADER};
use crate::{ut, ALL_ELM};
use iced_x86::{Decoder, DecoderOptions, Instruction};
use std::{io, ptr};
use winapi::shared::minwindef::{FALSE, LPVOID};
use winapi::shared::ntdef::HANDLE;
use winapi::um::dbghelp::SymCleanup;
use winapi::um::handleapi::CloseHandle;
use winapi::um::memoryapi::*;
use winapi::um::minwinbase::DEBUG_EVENT;
use winapi::um::processthreadsapi::{GetThreadContext, OpenThread, SetThreadContext};
use winapi::um::winbase::{Wow64GetThreadContext, Wow64SetThreadContext};
use winapi::um::winnt::{
    CONTEXT, CONTEXT_ALL, PAGE_EXECUTE_READWRITE, THREAD_ALL_ACCESS, WOW64_CONTEXT,
    WOW64_CONTEXT_ALL,
};
use crate::command::breakpoint::Brkpts;
use crate::ut::fmt::{print_lg, LevelPrint};

pub unsafe fn restore_byte_of_brkpt(h_proc: HANDLE, b_addr: u64, last_oc: u8) {
    let mut old_protect = 0;
    let mut written = 0;
    if VirtualProtectEx(h_proc, b_addr as LPVOID, 1, PAGE_EXECUTE_READWRITE, &mut old_protect) == 0 {
        print_lg(LevelPrint::Error, format!("error when changing memory protection at address : {:#x}", b_addr));
        return;
    }
    if WriteProcessMemory(h_proc, b_addr as LPVOID, ptr::addr_of!(last_oc) as LPVOID, 1, &mut written) == 0 {
        print_lg(LevelPrint::Error, format!("error when writing to memory at address : {:#x} : {}", b_addr, io::Error::last_os_error()));
    }
    if VirtualProtectEx(h_proc, b_addr as LPVOID, 1, old_protect, &mut old_protect) == 0 {
        print_lg(LevelPrint::Error, format!("error while restoring memory protection at address: {:#x}", b_addr));
    }
}



pub unsafe fn handle_br(h_proc: HANDLE, debug_event: DEBUG_EVENT, b_addr: u64, origin_b: u8, c_dbg: &mut DbgState) {
    print_lg(LevelPrint::Debug, format!("Breakpoint hit at address: {:#x}", b_addr));
    restore_byte_of_brkpt(h_proc, b_addr, origin_b);

    let mut h_thread = OpenThread(THREAD_ALL_ACCESS, FALSE, debug_event.dwThreadId);
    if h_thread.is_null() {
        print_lg(LevelPrint::Error, format!("Failed to open thread: {}", io::Error::last_os_error()));
        return;
    }

    match NT_HEADER {
        Some(NtHeaders::Headers64(_)) => {
            match ut::mem::alloc_size_align::<CONTEXT>() {
                Ok(pctx) => {
                    (*pctx).ContextFlags = CONTEXT_ALL;
                    if GetThreadContext(h_thread, pctx) != 0 {
                        (*pctx).Rip -= 1;
                        dbg_cmd::x64::cmd_wait(&mut *pctx, h_proc, &mut h_thread, c_dbg);
                        if SetThreadContext(h_thread, pctx) == 0 {
                            print_lg(LevelPrint::Error, format!("error when setting thread context: {}", io::Error::last_os_error()));
                        }
                    } else {
                        print_lg(LevelPrint::Critical1(debug_event.dwProcessId, c_dbg), format!("failed to get thread context: {}", io::Error::last_os_error()));
                    }
                }
                Err(e) => print_lg(LevelPrint::Critical1(debug_event.dwProcessId, c_dbg), e),
            }
        }
        Some(NtHeaders::Headers32(_)) => {
            match ut::mem::alloc_size_align::<WOW64_CONTEXT>() {
                Ok(pctx) => {
                    (*pctx).ContextFlags = WOW64_CONTEXT_ALL;
                    if Wow64GetThreadContext(h_thread, pctx) != 0 {
                        (*pctx).Eip -= 1;
                        dbg_cmd::x32::cmd_wait32(&mut *pctx, h_proc, &mut h_thread, c_dbg);
                        if Wow64SetThreadContext(h_thread, pctx) == 0 {
                            print_lg(LevelPrint::Error, format!("error when setting thread context: {}", io::Error::last_os_error()));
                        }
                    } else {
                        print_lg(LevelPrint::Critical1(debug_event.dwProcessId, c_dbg), format!("failed to get thread context: {}", io::Error::last_os_error()));
                    }
                }
                Err(e) => print_lg(LevelPrint::Critical1(debug_event.dwProcessId, c_dbg), e)
            }
        }
        None => print_lg(LevelPrint::Critical1(debug_event.dwProcessId, c_dbg), "Unexpected state: NT_HEADER is None"),
    }

    if *c_dbg == DbgState::Continue {
        let mut b_insn = [0u8; 15];
        if ReadProcessMemory(h_proc, b_addr as LPVOID, b_insn.as_mut_ptr() as LPVOID, 15, &mut 0) == 0 {
            print_lg(LevelPrint::Error, format!("Failed to get insn at address {:#x} : {}", b_addr, io::Error::last_os_error()));
            return;
        }
        let mut decoder = Decoder::with_ip(NT_HEADER.unwrap().get_bitness() as u32, &b_insn, b_addr, DecoderOptions::NONE);
        let mut insn = Instruction::new();
        decoder.decode_out(&mut insn);
        let next_addr = b_addr + insn.len() as u64;
        let mut last_oc = 0u8;
        if ReadProcessMemory(h_proc, next_addr as LPVOID, ptr::addr_of_mut!(last_oc) as LPVOID, 1, &mut 0) == 0 {
            print_lg(LevelPrint::Error, format!("Failed to read memory at address {:#x} : {}", next_addr, io::Error::last_os_error()));
            return;
        }
        (*&raw mut ALL_ELM).after_b.push(AfterB {
            last_addr_b: b_addr,
            after_b: next_addr,
            last_oc,
        });
        if WriteProcessMemory(h_proc, next_addr as LPVOID, &0xccu8 as *const u8 as LPVOID, 1, &mut 0) == 0 {
            print_lg(LevelPrint::Error, format!("Failed to write memory at address {:#x} : {}", next_addr, io::Error::last_os_error()));
            return;
        }
    }
    CloseHandle(h_thread);
}

pub unsafe fn set_breakpoint(h_proc: HANDLE, b_addr: u64, last_oc: &mut u8) -> Result<(), String>{
    let mut old_protect = 0;
    if VirtualProtectEx(h_proc, b_addr as LPVOID, 1, PAGE_EXECUTE_READWRITE, &mut old_protect) == 0 {
        return Err(format!("Failed to change memory protection at address: 0x{:x} : {}", b_addr, io::Error::last_os_error()));
    }
    if ReadProcessMemory(h_proc, b_addr as LPVOID, ptr::addr_of_mut!(*last_oc) as LPVOID, 1, ptr::null_mut()) == 0 {
        return Err(format!("Failed to read memory at address: 0x{:x} : {}", b_addr, io::Error::last_os_error()));
    }
    if WriteProcessMemory(h_proc, b_addr as LPVOID, &0xccu8 as *const u8 as LPVOID, 1, &mut 0) == 0 {
        return Err(format!("Failed to write breakpoint at address: 0x{:x} : {}", b_addr, io::Error::last_os_error()));
    }
    if VirtualProtectEx(h_proc, b_addr as LPVOID, 1, old_protect, &mut old_protect) == 0 {
        return Err(format!("Failed to restore memory protection at address: 0x{:x} : {}", b_addr, io::Error::last_os_error()))
    }
    print_lg(LevelPrint::Debug, format!("Breakpoint set at address: {:#x} in memory", b_addr));
    Ok(())
}



pub unsafe fn set_breakpoint_in_ret_func(h_proc: HANDLE, debug_event: DEBUG_EVENT, b: &mut Brkpts) {
    restore_byte_of_brkpt(h_proc, b.addr, b.origin_b);
    let h_thread = OpenThread(THREAD_ALL_ACCESS, FALSE, debug_event.dwThreadId);
    if h_thread.is_null() {
        print_lg(LevelPrint::Error, format!("Failed to open thread : {}", io::Error::last_os_error()));
        return;
    }
    (*&raw mut ST_FRAME).clear();
    let rip = match NT_HEADER.unwrap() {
        NtHeaders::Headers32(_) => {
            match ut::mem::alloc_size_align::<WOW64_CONTEXT>() {
                Ok(pctx) => {
                    (*pctx).ContextFlags = WOW64_CONTEXT_ALL;
                    if Wow64GetThreadContext(h_thread, pctx) == 0 {
                        print_lg(LevelPrint::Error, format!("Failed to get thread context: {}", io::Error::last_os_error()));
                        return;
                    }
                    (*pctx).Eip -= 1;
                    get_frame_st32(h_proc, h_thread, *pctx);
                    if Wow64SetThreadContext(h_thread, pctx) == 0 {
                        print_lg(LevelPrint::Error, format!("Failed to adjust EIP : {}", io::Error::last_os_error()));
                        return;
                    }
                    (*pctx).Eip as u64
                }
                Err(e) => {
                    print_lg(LevelPrint::Error, e);
                    return;
                },
            }
        }
        NtHeaders::Headers64(_) => {
            match ut::mem::alloc_size_align::<CONTEXT>() {
                Ok(pctx) => {
                    (*pctx).ContextFlags = CONTEXT_ALL;
                    if GetThreadContext(h_thread, pctx) == 0 {
                        print_lg(LevelPrint::Error, format!("Failed to get thread context: {}", io::Error::last_os_error()));
                        return;
                    }
                    (*pctx).Rip -= 1;
                    get_frame_st(h_proc, h_thread, *pctx);
                    if SetThreadContext(h_thread, pctx) == 0 {
                        print_lg(LevelPrint::Error, format!("Failed to adjust RIP : {}", io::Error::last_os_error()));
                        return;
                    }
                    (*pctx).Rip
                }
                Err(e) => {
                    print_lg(LevelPrint::Error, e);
                    return;
                }
            }
        }
    };

    if let Some(frame) = get_real_frame(rip) {
        let mut bh = Brkpts::from_addr_no_start(frame.AddrReturn.Offset);
        if let Err(e) = set_breakpoint(h_proc, bh.addr, &mut bh.origin_b) {
            print_lg(LevelPrint::Error, e);
            return;
        }
        print_lg(LevelPrint::Debug, format!("Address of function return: {:#x}", frame.AddrReturn.Offset));
        bh.b_mod = b.b_mod;
        (*ptr::addr_of_mut!(ALL_ELM)).break_va.push(bh);
    } else {
        print_lg(LevelPrint::Error, format!("Failed to retrieve function frame for instruction at {:#x}", b.addr - BASE_ADDR));
    }
    SymCleanup(h_proc);
    CloseHandle(h_thread);
}
