use std::ptr;
use winapi::um::winnt::HANDLE;
use crate::cli::All;
use crate::pefile::function;
use crate::symbol::{SymbolType, Symbols, SYMBOLS_V};
use crate::{pefile, symbol, usage, ALL_ELM};
use crate::command::breakpoint::Brkpts;
use crate::dbg::{memory, BASE_ADDR};
use crate::ut::fmt::{print_lg, LevelPrint};

fn clear_symbols() {
    unsafe {
        *SYMBOLS_V = Symbols::default();
        symbol::IMAGE_BASE = 0;
        pefile::NT_HEADER = None;
    }
}

fn restore_breakpoints(h_proc: HANDLE, breakpoints: &Vec<Brkpts>, offset: u64) {
    for b in breakpoints {
        unsafe {
            memory::breakpoint::restore_byte_of_brkpt(h_proc, b.addr + offset, b.origin_b);
        }
    }
}

fn print_reset_message(item: &str) {
    print_lg(LevelPrint::DebugO, format!("all {item} have been cleared"));
}

pub fn handle_reset(linev: &[&str]) {
    if linev.len() != 2 {
        eprintln!("{}", usage::USAGE_RESET);
        return;
    }
    let binding = linev[1].to_lowercase();
    let opt = binding.trim();
    unsafe {
        let p_allm = ptr::addr_of_mut!(ALL_ELM);
        match opt {
            "file" => {
                clear_symbols();
                (*p_allm).file = None;
                print_reset_message("file context");
            }
            "breakpoint" | "b" => {
                (*p_allm).break_rva.clear();
                print_reset_message("rva breakpoints");
            }
            "symbol" | "s" => {
                clear_symbols();
                print_reset_message("symbols");
            }
            "hook" | "ho" => {
                (*p_allm).hook.clear();
                print_reset_message("hooks");
            }
            "break-va" | "b-va" => {
                (*p_allm).break_va.clear();
                print_reset_message("va breakpoints");
            }
            "break-ret" | "b-ret" => {
                (*p_allm).break_ret.clear();
                print_reset_message("function returns");
            }
            "skip" => {
                (*p_allm).skip_addr.clear();
                print_reset_message("skipped functions");
            }
            "args" | "arg" | "argv" => {
                (*p_allm).arg = None;
                print_reset_message("arguments");
            }
            "watchpoint" | "watchpts" | "w" => {
                (*p_allm).watchpts.clear();
                print_reset_message("watchpoints");
            }
            "all" => {
                clear_symbols();
                (*p_allm).skip_addr.clear();
                (*p_allm).break_ret.clear();
                (*ptr::addr_of_mut!(function::FUNC_INFO)).clear();
                (*p_allm).hook.clear();
                *(*p_allm) = All::default();
                SYMBOLS_V.symbol_type = SymbolType::Un;
                (*&raw mut SYMBOLS_V).symbol_file.clear();
                print_reset_message("elements");
            }
            _ => eprintln!("{}", usage::USAGE_RESET),
        }
    }
}

pub fn reset_proc(linev: &[&str], h_proc: HANDLE) {
    if linev.len() != 2 {
        eprintln!("{}", usage::USAGE_RESET);
        return;
    }
    let binding = linev[1].to_lowercase();
    let opt = binding.trim();

    unsafe {
        let p_allm = ptr::addr_of_mut!(ALL_ELM);
        match opt {
            "file" => {
                clear_symbols();
                (*p_allm).file = None;
                print_reset_message("file context");
            }
            "breakpoint" | "b" | "break" => {
                restore_breakpoints(h_proc, &(*p_allm).break_rva, BASE_ADDR);
                (*p_allm).break_rva.clear();
                print_reset_message("breakpoints");
            }
            "break-va" | "b-va" => {
                restore_breakpoints(h_proc, &(*p_allm).break_va, 0);
                (*p_allm).break_va.clear();
                print_reset_message("va breakpoints");
            }
            "symbol" | "s" => {
                clear_symbols();
                print_reset_message("symbols");
            }
            "hook" | "ho" => {
                restore_breakpoints(h_proc, &(*p_allm).hook.iter().map(|h| {
                    let mut b = Brkpts::default();
                    b.addr = h.target;
                    b.origin_b = h.origin_byte;
                    b
                }).collect::<Vec<Brkpts>>(), BASE_ADDR);
                (*p_allm).hook.clear();
                print_reset_message("hooks");
            }
            "break-ret" | "b-ret" => {
                (*p_allm).break_ret.clear();
                print_reset_message("function returns");
            }
            "skip" => {
                restore_breakpoints(h_proc, &(*p_allm).skip_addr, BASE_ADDR);
                (*p_allm).skip_addr.clear();
                print_reset_message("skipped functions");
            }
            "args" | "arg" | "argv" => {
                (*p_allm).arg = None;
                print_reset_message("arguments");
            }
            "watchpoint" | "watchpts" | "w" => {
                (*p_allm).watchpts.clear();
                print_reset_message("watchpoints");
            }
            "all" => {
                clear_symbols();
                restore_breakpoints(h_proc, &(*p_allm).skip_addr, BASE_ADDR);
                restore_breakpoints(h_proc, &(*p_allm).break_ret, BASE_ADDR);
                restore_breakpoints(h_proc, &(*p_allm).hook.iter().map(|h| {
                    let mut b = Brkpts::default();
                    b.addr = h.target;
                    b.origin_b = h.origin_byte;
                    b
                }).collect::<Vec<Brkpts>>(), BASE_ADDR);
                restore_breakpoints(h_proc, &(*p_allm).break_va, 0);
                restore_breakpoints(h_proc, &(*p_allm).break_rva, BASE_ADDR);
                *ALL_ELM = All::default();
                print_reset_message("elements");
            }
            _ => eprintln!("{}", usage::USAGE_RESET),
        }
    }
}
