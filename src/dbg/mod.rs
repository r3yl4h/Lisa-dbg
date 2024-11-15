use std::{io, ptr};
use winapi::um::debugapi::DebugActiveProcessStop;

use winapi::um::winnt::{CONTEXT, HANDLE, WOW64_CONTEXT};
use crate::cli::ALL_ELM;
use crate::ut::fmt::{print_lg, LevelPrint};

pub mod attach;
pub mod dbg_cmd;
mod exec;
mod handle_point;
pub mod memory;

const STATUS_WX86_BREAKPOINT: u32 = 0x4000001f;
const STATUS_WX86_SINGLE_STEP: u32 = 0x4000001e;


#[derive(PartialOrd, PartialEq, Copy, Clone)]
pub enum DbgState {
    Continue,
    NeedStop,
    Stopped,
}



pub trait RealAddr {
    fn real_addr64(&self, ctx: CONTEXT) -> u64;
    fn real_addr32(&self, ctx: WOW64_CONTEXT) -> u32;
    fn real_addr(&self, ctx: *const CONTEXT) -> u64;
}

pub static mut BASE_ADDR: u64 = 0;

pub fn run() {
    unsafe {
        if let Some(file) = &(*ptr::addr_of!(ALL_ELM)).file {
            let arg = {
                if let Some(arg) = &(*ptr::addr_of!(ALL_ELM)).arg {
                    format!("{} {}", file, arg)
                } else {
                    file.to_string()
                }
            };
            exec::start_debugging(&arg);
        } else {
            print_lg(LevelPrint::ErrorO, "Please enter a file path");
        }
    }
}


pub unsafe fn stop_dbg(pid: u32) {
    for crt_func in (*ptr::addr_of_mut!(ALL_ELM)).crt_func.iter_mut() {
        crt_func.addr = 0;
    }
    BASE_ADDR = 0;
    if DebugActiveProcessStop(pid) == 0 {
        print_lg(LevelPrint::Error, format!("failed to DebugActiveProcessStop : {}", io::Error::last_os_error()))
    }
}


fn init(h_proc: HANDLE) {
    unsafe {
        for hook in &mut (*ptr::addr_of_mut!(ALL_ELM)).hook {
            if let Err(e) = memory::breakpoint::set_breakpoint(h_proc, hook.target + BASE_ADDR, &mut hook.origin_byte) {
                print_lg(LevelPrint::Error, e);
            }
        }

        for addr in &mut (*ptr::addr_of_mut!(ALL_ELM)).break_rva {
            if let Err(e) = memory::breakpoint::set_breakpoint(h_proc, addr.addr + BASE_ADDR, &mut addr.origin_b) {
                print_lg(LevelPrint::Error, e);
            }
        }

        for addr in &mut (*ptr::addr_of_mut!(ALL_ELM)).break_ret {
            if let Err(e) = memory::breakpoint::set_breakpoint(h_proc, addr.addr + BASE_ADDR, &mut addr.origin_b) {
                print_lg(LevelPrint::Error, e);
            }
        }

        for addr in &mut (*ptr::addr_of_mut!(ALL_ELM)).break_ret_va {
            if let Err(e) = memory::breakpoint::set_breakpoint(h_proc, addr.addr, &mut addr.origin_b) {
                print_lg(LevelPrint::Error, e);
            }
        }

        for addr_over in &mut (*ptr::addr_of_mut!(ALL_ELM)).skip_addr {
            if let Err(e) = memory::set_addr_over(h_proc, addr_over.addr + BASE_ADDR, &mut addr_over.origin_b) {
                print_lg(LevelPrint::Error, e);
            }
        }

        for crt in (*ptr::addr_of_mut!(ALL_ELM)).crt_func.iter_mut() {
            if let Err(e) = memory::func::set_cr_function(h_proc, crt) {
                print_lg(LevelPrint::Error, e);
            }
        }
    }
}