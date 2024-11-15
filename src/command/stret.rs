use std::ptr;
use std::str::FromStr;
use crate::dbg::{memory, BASE_ADDR};
use crate::usage::USAGE_B_RET_VA;
use winapi::shared::ntdef::HANDLE;
use crate::cli::ALL_ELM;
use crate::command::breakpoint::Brkpts;
use crate::usage;
use crate::ut::fmt::{print_lg, LevelPrint};

pub fn st_return(linev: &[&str]) {
    if linev.len() < 2 {
        print_lg(LevelPrint::ErrorO, usage::USAGE_B_RET);
        return;
    }
    
    match Brkpts::from_str(linev[0]) {
        Ok(b) => unsafe {
            if (*ptr::addr_of!(ALL_ELM)).break_contain(b.addr) {
                print_lg(LevelPrint::ErrorO, format!("you have already placed a breakpoint here {:#x}", b.addr));
                return;
            }

            (*ptr::addr_of_mut!(ALL_ELM)).break_ret.push(b);
            print_lg(LevelPrint::DebugO, format!("a breakpoint will be placed at the return address of the function containing the instruction at address {:#x} + base addr", b.addr));
        }
        Err(e) => print_lg(LevelPrint::ErrorO, e),
    }
}

pub fn handle_stret(linev: &[&str], h_proc: HANDLE) {
    if linev.len() > 2 {
        match Brkpts::from_str(&linev.join(" ")) {
            Ok(mut b) => unsafe {
                if (*ptr::addr_of_mut!(ALL_ELM)).break_contain(b.addr) {
                    print_lg(LevelPrint::ErrorO, format!("you have already placed a breakpoint here {:#x}", b.addr));
                    return;
                }
                if let Err(e) = memory::breakpoint::set_breakpoint(h_proc, b.addr + BASE_ADDR, &mut b.origin_b) {
                    print_lg(LevelPrint::ErrorO, e);
                }
                (*ptr::addr_of_mut!(ALL_ELM)).break_ret.push(b);
                print_lg(LevelPrint::DebugO, format!("Breakpoint set at address {:#x}", b.addr));
            }
            Err(e) => print_lg(LevelPrint::ErrorO, e),
        }
    } else {
        print_lg(LevelPrint::DebugO, usage::USAGE_B_RET);
    }
}

pub fn handle_b_ret_va(linev: &[&str]) {
    if linev.len() < 2 {
        print_lg(LevelPrint::DebugO, USAGE_B_RET_VA);
        return;
    }
    match Brkpts::from_str(&linev.join(" ")) {
        Ok(b) => unsafe {
            if (*ptr::addr_of!(ALL_ELM)).break_contain(b.addr) {
                print_lg(LevelPrint::ErrorO, format!("you have already placed a breakpoint here {:#x}", b.addr));
                return;
            }
            (*ptr::addr_of_mut!(ALL_ELM)).break_ret_va.push(b);
            print_lg(LevelPrint::DebugO, format!("a breakpoint will be placed at the return address of the function containing the instruction at address {:#x}", b.addr));
        }
        Err(e) => print_lg(LevelPrint::ErrorO, e),
    }
}

pub fn handle_proc_b_ret_va(linev: &[&str], h_proc: HANDLE) {
    if linev.len() < 2 {
        print_lg(LevelPrint::DebugO, USAGE_B_RET_VA);
        return;
    }
    
    match Brkpts::from_str(&linev.join(" ")) {
        Ok(mut b) => unsafe {
            if (*ptr::addr_of!(ALL_ELM)).break_contain(b.addr) {
                print_lg(LevelPrint::ErrorO, format!("you have already placed a breakpoint here {:#x}", b.addr));
                return;
            }
            if let Err(e) = memory::breakpoint::set_breakpoint(h_proc, b.addr, &mut b.origin_b){
                print_lg(LevelPrint::ErrorO, e);
                return;
            }
            (*ptr::addr_of_mut!(ALL_ELM)).break_ret_va.push(b);
            print_lg(LevelPrint::DebugO, format!("a breakpoint will be placed at the return address of the function containing the instruction at address {:#x}", b.addr));
        }
        Err(e) => print_lg(LevelPrint::ErrorO, e),
    }
}
