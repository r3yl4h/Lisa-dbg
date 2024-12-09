use std::ptr;
use std::str::FromStr;
use crate::dbg::memory::breakpoint::set_breakpoint;
use crate::dbg::{memory, BASE_ADDR};
use crate::usage;
use winapi::shared::ntdef::HANDLE;
use winapi::um::winnt::CONTEXT;
use crate::cli::ALL_ELM;
use crate::ut::{fmt::*};
use crate::ut::*;


#[derive(Debug, Copy, Clone)]
pub enum BMOD {
    Normally,
    Pro
}


impl Default for BMOD {
    fn default() -> Self { BMOD::Normally }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct Brkpts {
    pub addr: u64,
    pub origin_b: u8,
    pub b_mod: BMOD
}





impl Brkpts {
    pub fn from_addr_no_start(addr: u64) -> Self {
        let mut res = Brkpts::default();
        res.addr = addr;
        res
    }
    
    pub fn from_str_ctx(s: &str, ctx: *const CONTEXT) -> Result<Self, StrErr> {
        let linev = s.split_whitespace().collect::<Vec<&str>>();
        if linev.len() < 2 {
            return Err(StrErr::ShortArg);
        }
        let mut result = Brkpts::default();
        match get_addr_va(linev[1], ctx) {
            Ok(addr) => result.addr = addr,
            Err(e) => return Err(StrErr::InvalidAddr(e.to_string()))
        }
        if linev.len() == 3 {
            match linev[2].to_lowercase().as_str() {
                "normal" | "normally" => result.b_mod = BMOD::Normally,
                "pro" => result.b_mod = BMOD::Pro,
                _ => return Err(StrErr::InvalidMod(linev[2].to_string()))
            }
        }
        Ok(result)
    }
}

pub enum StrErr {
    ShortArg,
    InvalidAddr(String),
    InvalidMod(String)
}


impl std::fmt::Display for StrErr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            StrErr::ShortArg => write!(f, "Short argument"),
            StrErr::InvalidAddr(e) => write!(f, "Invalid specified address : {e}"),
            StrErr::InvalidMod(e) => write!(f, "Invalid breakpoints Mod : {e}"),
        }
    }
}



impl FromStr for Brkpts {
    type Err = StrErr;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let linev = s.split_whitespace().collect::<Vec<&str>>();
        if linev.len() < 2 {
            return Err(StrErr::ShortArg);
        }
        let mut result = Brkpts::default();
        match get_addr_br(linev[1]) {
            Ok(addr) => result.addr = addr,
            Err(e) => return Err(StrErr::InvalidAddr(e.to_string()))
        }
        if linev.len() == 3 {
            match linev[2].to_lowercase().as_str() {
                "normal" | "normally" => result.b_mod = BMOD::Normally,
                "pro" => result.b_mod = BMOD::Pro,
                _ => return Err(StrErr::InvalidMod(linev[2].to_string()))
            }
        }
        Ok(result)
    }
}

pub fn handle_breakpts(linev: &[&str]) {
    if linev.len() == 1 {
        eprintln!("{}", usage::USAGE_BRPT);
        return;
    }
    match Brkpts::from_str(&linev.join(" ")) {
        Ok(b) => unsafe {
            let p_allm = ptr::addr_of_mut!(ALL_ELM);
            if (*p_allm).break_contain(b.addr) {
                print_lg(LevelPrint::Error, format!("you have already placed a breakpoint here {:#x}", b.addr));
                return;
            }
            (*p_allm).break_rva.push(b);
        }
        Err(e) => print_lg(LevelPrint::ErrorO, format!("failed to set breakpoint : {}", e)),
    } 
}



pub fn handle_break_va(linev: &[&str]) {
    if linev.len() != 2 {
        println!("b-va <va>");
        return;
    }
    
    match Brkpts::from_str(&linev.join(" ")) {
        Ok(b) => unsafe {
            if (*ptr::addr_of!(ALL_ELM)).break_contain(b.addr) {
                print_lg(LevelPrint::ErrorO, format!("you have already placed a breakpoint here {:#x}", b.addr));
                return;
            }
            (*ptr::addr_of_mut!(ALL_ELM)).break_va.push(b);
            print_lg(LevelPrint::DebugO, format!("breakpoints are set at address {:#x}", b.addr));
        }
        Err(e) => print_lg(LevelPrint::ErrorO, format!("failed to set breakpoint : {}", e)),
    }
}





pub fn handle_b_va_proc(linev: &[&str], h_proc: HANDLE, ctx: *const CONTEXT) {
    if linev.len() != 2 {
        println!("b-va <address>");
        return;
    }
    
    unsafe {
        match Brkpts::from_str_ctx(&linev.join(" "), ctx) {
            Ok(mut b) => {
                if (*ptr::addr_of!(ALL_ELM)).break_contain(b.addr) {
                    print_lg(LevelPrint::ErrorO, format!("you have already placed a breakpoint here : {:#x}", b.addr));
                    return;
                }
                if let Err(e) = set_breakpoint(h_proc, b.addr, &mut b.origin_b) {
                    print_lg(LevelPrint::ErrorO, e);
                }else {
                    (*ptr::addr_of_mut!(ALL_ELM)).break_va.push(b);
                }
            }
            Err(e) => print_lg(LevelPrint::ErrorO, format!("failed to set breakpoint : {}", e)),
        }
    }
}


pub fn handle_breakpoint_proc(linev: &[&str], h_proc: HANDLE) {
    if linev.len() != 2 {
        eprintln!("{}", usage::USAGE_BRPT);
    } else {
        match Brkpts::from_str(&linev.join(" ")) {
            Ok(mut b) => unsafe {
                if (*ptr::addr_of!(ALL_ELM)).break_contain(b.addr) {
                    print_lg(LevelPrint::ErrorO, format!("you have already placed a breakpoint here : {:#x}", b.addr));
                    return;
                }
                if let Err(e) = set_breakpoint(h_proc, b.addr + BASE_ADDR, &mut b.origin_b) {
                    print_lg(LevelPrint::ErrorO, e);
                }else {
                    (*ptr::addr_of_mut!(ALL_ELM)).break_rva.push(b);
                }
            }
            Err(e) => print_lg(LevelPrint::ErrorO, format!("failed to set breakpoint : {}", e)),
        }
    }
}


pub fn handle_restore_breakpoint_proc(linev: &[&str], h_proc: HANDLE) {
    if linev.len() == 2 {
        let addr_str = linev[1];
        let addr = match get_addr_br(addr_str) {
            Ok(value) => value,
            Err(e) => {
                print_lg(LevelPrint::Error, e);
                return;
            }
        };
        
        if let Some(b) = unsafe {(*ptr::addr_of!(ALL_ELM)).find_b_rva_with_addr(addr)} {
            unsafe { memory::breakpoint::restore_byte_of_brkpt(h_proc, b.addr + BASE_ADDR, b.origin_b) }
        }
    }
}
