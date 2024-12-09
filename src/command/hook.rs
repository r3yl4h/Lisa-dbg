use std::ptr;
use winapi::um::winnt::HANDLE;
use crate::{usage, ALL_ELM};
use crate::dbg::{memory, BASE_ADDR};
use crate::symbol::SYMBOLS_V;
use crate::ut::cast::str_to;
use crate::ut::fmt::{print_lg, LevelPrint};

#[derive(Debug, Default, Copy, Clone)]
pub struct Hook {
    pub target: u64,
    pub replacen: u64,
    pub origin_byte: u8,
}

fn get_addr_or_symbol(linev: &[&str], idx: usize) -> Option<u64> {
    match str_to::<u64>(linev[idx]) {
        Ok(addr) => Some(addr),
        Err(_) => {
            if let Some(sym) = unsafe { (*ptr::addr_of!(SYMBOLS_V)).symbol_file.iter().find(|s| s.name == linev[idx]) } {
                if sym.offset > 0 {
                    Some(sym.offset as u64)
                } else {
                    print_lg(LevelPrint::ErrorO, "You cannot specify local symbols");
                    None
                }
            } else {
                print_lg(LevelPrint::ErrorO, format!("Invalid target: {}", linev[idx]));
                None
            }
        }
    }
}


fn set_hook(addr1: u64, addr2: u64, h_proc: Option<HANDLE>) {
    let mut orig_b = 0;
    if let Some(h_proc) = h_proc {
        if let Err(e) = unsafe { memory::breakpoint::set_breakpoint(h_proc, addr1 + BASE_ADDR, &mut orig_b) }  {
            print_lg(LevelPrint::ErrorO, format!("Failed to set hoot : {}", e));
            return;
        }
    }
    unsafe { (*ptr::addr_of_mut!(ALL_ELM)).hook.push(Hook { target: addr1, replacen: addr2, origin_byte: orig_b }); }
    print_lg(LevelPrint::DebugO, format!("Now when the program reaches rva {:#x}, it will be redirected to rva {:#x}", addr1, addr2));
}



pub fn hook(linev: &[&str]) {
    if linev.len() < 3 {
        eprintln!("{}", usage::USAGE_HOOK);
        return;
    }

    let addr1 = match get_addr_or_symbol(linev, 1) {
        Some(addr) => addr,
        None => return,
    };

    let addr2 = match get_addr_or_symbol(linev, 2) {
        Some(addr) => addr,
        None => return,
    };

    set_hook(addr1, addr2, None);
}



pub fn handle_hook_proc(linev: &[&str], h_proc: HANDLE) {
    if linev.len() < 3 {
        eprintln!("{}", usage::USAGE_HOOK);
        return;
    }

    let addr1 = match get_addr_or_symbol(linev, 1) {
        Some(addr) => addr,
        None => return,
    };

    let addr2 = match get_addr_or_symbol(linev, 2) {
        Some(addr) => addr,
        None => return,
    };

    set_hook(addr1, addr2, Some(h_proc));
}
