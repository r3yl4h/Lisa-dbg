use std::ptr;
use crate::command::info;
use crate::dbg::dbg_cmd::usages;
use crate::dbg::memory::stack::LEN;
use crate::dbg::RealAddr;
use crate::symbol::SYMBOLS_V;
use crate::usage::USAGE_SYM_INFO;
use winapi::um::winnt::CONTEXT;
use crate::ut::fmt::*;

pub fn handle_sym_addr(linev: &[&str], ctx: *const CONTEXT) {
    if linev.len() != 2 {
        println!("{}", usages::USAGE_SA);
        return;
    }
    let name = linev[1];
    unsafe {
        if let Some(sym) = (*ptr::addr_of!(SYMBOLS_V)).symbol_file.iter().find(|s| s.name == name) {
            print_lg(LevelPrint::DebugO, format!("the address of {name} is {:#x}", sym.real_addr(ctx)));
        } else {
            print_lg(LevelPrint::ErrorO, format!("the symbol {name} is unknow"));
        }
    }
}

pub fn handle_sym_info(linev: &[&str], ctx: *const CONTEXT) {
    if linev.len() == 1 {
        println!("{USAGE_SYM_INFO}");
        return;
    }

    let sym_name = linev[1];
    if let Some(sym) = unsafe { (*ptr::addr_of_mut!(SYMBOLS_V)).symbol_file.iter().find(|s| s.name == sym_name) } {
        println!(
            "    {}name    : {}\
            \n    {}Address : {:#x}\
            \n    {}Type    : {}\
            \n    {}Size    : {:#x}\
            \n    {}File    : {}:{}\
            {RESET_COLOR}\n",
            GREEN_COL, sym.name,
            ADDR_COLOR, sym.real_addr(ctx),
            BLUE_COLOR, sym.types_e,
            MAGENTA, sym.size,
            WAR_COLOR, sym.filename, sym.line,
        );
    } else {
        print_lg(LevelPrint::ErrorO, "The name of the symbol is unknown");
    }
}

pub fn print_local_sym(ctx: *const CONTEXT) {
    unsafe {
        let psym = ptr::addr_of_mut!(SYMBOLS_V);
        let temp_sym = (*psym).symbol_file.clone();
        (*psym).symbol_file.reverse();
        (*psym).symbol_file.truncate(LEN);
        info::print_sym(&["s"], ctx);
        (*psym).symbol_file = temp_sym;
    }
}
