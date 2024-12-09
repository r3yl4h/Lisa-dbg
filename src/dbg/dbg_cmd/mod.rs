use crate::ctx_ptr;
use crate::dbg::memory::stack::ST_FRAME;
use crate::dbg::{memory, DbgState, RealAddr, BASE_ADDR};
use crate::pefile::function::FUNC_INFO;
use crate::pefile::NT_HEADER;
use crate::symbol::{SymbolType, SYMBOLS_V};
use crate::{command, usage, ALL_ELM};
use std::io;
use std::io::Write;
use std::str::FromStr;
use winapi::shared::ntdef::HANDLE;
use winapi::um::dbghelp::SymCleanup;
use winapi::um::winbase::DebugSetProcessKillOnExit;
use winapi::um::winnt::CONTEXT;
use crate::command::breakpoint::Brkpts;
use crate::ut::cast::{str_to, NumConvert};
use crate::ut::fmt::*;

pub(crate) mod disasm;
pub mod usages;
pub mod x32;
pub mod x64;

pub(crate) fn init_cm(ctx: CONTEXT, h_proc: HANDLE, h_thread: HANDLE, addr_func: &mut u64) {
    unsafe {
        memory::stack::LEN = 0;
        (*&raw mut ST_FRAME).clear();
        memory::stack::get_frame_st(h_proc, h_thread, ctx);
        *addr_func = if let Some(func) = (*&raw const FUNC_INFO).iter().find(|f| {
            f.BeginAddress as u64 + BASE_ADDR <= ctx.Rip
                && f.EndAddress as u64 + BASE_ADDR >= ctx.Rip
        }) {
            func.BeginAddress as u64 + BASE_ADDR
        } else {
            ctx.Rip
        };
        if SYMBOLS_V.symbol_type == SymbolType::PDB {
            memory::stack::get_local_sym(h_proc, *addr_func, ctx_ptr!(ctx));
        } else {
            SymCleanup(h_proc);
        }
    }
}

#[macro_export]
macro_rules! ctx_ptr {
    ($wow64_ctx:expr) => {
        std::ptr::addr_of!($wow64_ctx) as *const CONTEXT
    };
}

fn unint_cm() {
    unsafe {
        for _ in 0..memory::stack::LEN {
            (*&raw mut SYMBOLS_V).symbol_file.pop();
        }
    }
}

fn handle_backtrace(linev: &[&str], ctx: *const CONTEXT) {
    let count;
    let arg1 = linev.get(1);
    if arg1 == Some(&"full") || arg1.is_none() {
        count = usize::MAX;
    } else {
        match str_to::<usize>(arg1.unwrap()) {
            Ok(counts) => count = counts,
            Err(e) => {
                print_lg(LevelPrint::ErrorO, format!("invalid count: {e}"));
                return;
            }
        }
    }
    command::info::print_frame(count, ctx);
}

fn print_curr_func(addr_func: u64, ctx: *const CONTEXT) {
    unsafe {
        println!("{}Function    : {:#x} {}{RESET_COLOR}",
            ADDR_COLOR, addr_func,
            if let Some(sym) = (*&raw const SYMBOLS_V).symbol_file.iter().find(|s| s.real_addr(ctx) == addr_func)
            {
                format!("<{}>", sym.name)
            } else {
                "".to_string()
            }
        );
        if let Some(func) = (*&raw const FUNC_INFO).iter().find(|f| f.BeginAddress as u64 + BASE_ADDR == addr_func) {
            println!("{}End Address : {:#x}", VALUE_COLOR, func.EndAddress as u64 + BASE_ADDR);
            println!("{}Size        : {:#x}{RESET_COLOR}", MAGENTA, func.EndAddress - func.BeginAddress);
        }
    }
}

fn handle_quit(input: &mut String, continue_debugging: &mut DbgState, stop_process: &mut bool) {
    input.clear();
    print!("Are you sure to stop this process? [y/n] : ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(input).unwrap();
    if input.trim() == "y" || input.trim() == "yes" {
        if unsafe {(*&raw mut ALL_ELM).attach.is_some()}{
            loop {
                print!("kill attach process ? [y/n] : ");
                input.clear();
                io::stdout().flush().unwrap();
                io::stdin().read_line(input).unwrap();
                let input = input.trim();
                if input == "n" || input == "nop" {
                    unsafe {
                        DebugSetProcessKillOnExit(0);
                    }
                    break;
                } else if input == "y" || input == "yes" {
                    unsafe {
                        DebugSetProcessKillOnExit(1);
                    }
                    break;
                } else {
                    print_lg(LevelPrint::ErrorO, "please choose a valid choice");
                    continue;
                }
            }
        }
        *continue_debugging = DbgState::NeedStop;
        *stop_process = true;
    }
}

fn handle_ret<T: NumConvert + num::Num + std::ops::SubAssign + std::fmt::LowerHex + Copy>(rip: &mut T, rsp: &mut T) {
    unsafe {
        if let Some(frame_ret) = memory::stack::get_real_frame(rip.to_u64()) {
            *rip = T::from_u64(frame_ret.AddrReturn.Offset);
            *rsp -= T::from_u64(NT_HEADER.unwrap().get_size_of_arch() as u64);
            println!(
                "{VALID_COLOR}now rip points to the address : {VALUE_COLOR}{:#x}{RESET_COLOR}\n\
                {VALID_COLOR}and rsp was decremented by {} : {VALUE_COLOR}{:#x}{RESET_COLOR}",
                *rip,
                NT_HEADER.unwrap().get_size_of_arch(),
                *rsp
            );
        } else {
            print_lg(LevelPrint::ErrorO, format!("an error occurred while getting return address of the current stack frame: rip: {:#x}", *rip));
        }
    }
}

fn handle_skip(linev: &[&str], h_proc: HANDLE) {
    if linev.len() == 2 {
        match Brkpts::from_str(&linev.join(" ")) {
            Ok(mut b) => unsafe {
                if let Err(e) = memory::set_addr_over(h_proc, b.addr + BASE_ADDR, &mut b.origin_b) {
                    print_lg(LevelPrint::ErrorO, e);
                }else {
                    (*&raw mut ALL_ELM).skip_addr.push(b);
                }
            }
            Err(e) => print_lg(LevelPrint::ErrorO, format!("error while trying to skip memory: {}", e)),
        }
    } else {
        println!("{}", usage::USAGE_SKIP);
    }
}
