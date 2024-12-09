use crate::dbg::memory::watchpoint;
use crate::dbg::{memory, RealAddr, BASE_ADDR};
use crate::symbol::SYMBOLS_V;
use crate::{usage, ALL_ELM};
use std::ptr;
use winapi::um::winnt::{CONTEXT, HANDLE};
use crate::ut::cast::str_to;
use crate::ut::fmt::{print_lg, LevelPrint};

pub fn remove_element(linev: &[&str]) {
    if linev.len() < 3 {
        print_lg(LevelPrint::ErrorO, usage::USAGE_REMOVE.to_string());
        return;
    }
    let element = linev[1];
    let target = linev[2];
    let binding = element.to_lowercase();
    let element = binding.as_str();
    let addr = match str_to::<i64>(target) {
        Ok(value) => value,
        Err(_) => unsafe {
            if let Some(sym) = (*ptr::addr_of!(SYMBOLS_V)).symbol_file.iter().find(|s| s.name == target) {
                sym.offset
            } else {
                0
            }
        },
    };

    let vec_option = unsafe {
        match element {
            "breakpoint" | "b" => Some(ptr::addr_of_mut!(ALL_ELM.break_rva)),
            "break-ret" | "b-ret" => Some(ptr::addr_of_mut!(ALL_ELM.break_ret)),
            "skip" => Some(ptr::addr_of_mut!(ALL_ELM.skip_addr)),
            "break-va" | "b-va" => Some(ptr::addr_of_mut!(ALL_ELM.break_va)),
            "break-ret-va" => Some(ptr::addr_of_mut!(ALL_ELM.break_ret_va)),
            "hook" => {
                if addr == 0 {
                    print_lg(LevelPrint::ErrorO, format!("invalid target: {target}"));
                    return;
                }
                (*ptr::addr_of_mut!(ALL_ELM)).hook.retain(|h| h.target != addr as u64);
                print_lg(LevelPrint::DebugO, format!("{target} has been retained successfully"));
                return;
            }
            "watchpoint" | "watch" | "w" => {
                if addr == 0 {
                    print_lg(LevelPrint::ErrorO, format!("invalid target: {target}"));
                    return;
                }
                (*ptr::addr_of_mut!(ALL_ELM)).watchpts.retain(|w| w.offset != addr);
                print_lg(LevelPrint::DebugO, format!("{target} has been retained successfully"));
                return;
            }
            "def" => {
                if linev.len() != 4 {
                    print_lg(LevelPrint::ErrorO, "Please specify a target for remove".to_string());
                    return;
                }
                let target_name = linev[3].to_string();
                match target {
                    "func" | "function" => (*ptr::addr_of_mut!(ALL_ELM)).crt_func.retain(|f| f.name != target_name),
                    "struct" => (*ptr::addr_of_mut!(ALL_ELM)).struct_def.retain(|s| s.get_name_of_struct() != target_name),
                    _ => {
                        print_lg(LevelPrint::ErrorO, format!("unknown element '{target}'"));
                        return;
                    }
                }
                print_lg(LevelPrint::DebugO, format!("{target_name} was retained successfully"));
                return;
            }
            "s" | "symbol" | "symbols" => {
                (*ptr::addr_of_mut!(SYMBOLS_V)).symbol_file.retain(|s|s.name != target);
                print_lg(LevelPrint::DebugO, format!("{target} was retained successfully"));
                return;
            }
            _ => None,
        }
    };

    unsafe {
        if let Some(vec_ptr) = vec_option {
            if addr == 0 {
                print_lg(LevelPrint::ErrorO, format!("invalid target: {target}"));
                return;
            }
            let vec = &mut *vec_ptr;
            if let Some(pos) = vec.iter().position(|e| e.addr == addr as u64) {
                vec.remove(pos);
                print_lg(LevelPrint::DebugO, format!("{target} has been retained successfully"));
            } else {
                print_lg(LevelPrint::ErrorO, format!("'{}' is not a valid target", element));
            }
        } else {
            print_lg(LevelPrint::ErrorO, format!("'{}' is not a valid target", element));
        }
    }
}



pub fn remove_element_proc(linev: &[&str], h_proc: HANDLE, ctx: &mut CONTEXT) {
    if linev.len() != 3 {
        print_lg(LevelPrint::ErrorO, usage::USAGE_REMOVE.to_string());
        return;
    }
    let element = linev[1];
    let target = linev[2];
    let addr = match str_to::<u64>(target) {
        Ok(value) => value,
        Err(_) => unsafe {
            if let Some(sym) = (*ptr::addr_of!(SYMBOLS_V)).symbol_file.iter().find(|s| s.name == target) {
                sym.real_addr64(*ctx)
            } else {
                print_lg(LevelPrint::ErrorO, format!("invalid target: {target}"));
                return;
            }
        },
    };
    let p_all_elm = ptr::addr_of_mut!(ALL_ELM);
    match element {
        "breakpoint" | "b" => unsafe {
            if let Some(pos) = (*p_all_elm).break_rva.iter().position(|&b| b.addr == addr) {
                let b = (*p_all_elm).break_rva.remove(pos);
                memory::breakpoint::restore_byte_of_brkpt(h_proc, b.addr + BASE_ADDR, b.origin_b);
            } else {
                print_lg(LevelPrint::ErrorO, format!("no breakpoint was set for this rva: {:#x}", addr));
            }
        },
        "break-va" | "b-va" => unsafe {
            if let Some(pos) = (*p_all_elm).break_va.iter().position(|&b| b.addr == addr) {
                let b = (*p_all_elm).break_rva.remove(pos);
                memory::breakpoint::restore_byte_of_brkpt(h_proc, b.addr + BASE_ADDR, b.origin_b);
            } else {
                print_lg(LevelPrint::ErrorO, format!("no breakpoint was set for this va: {:#x}", addr));
            }
        },
        "break-ret" | "b-ret" => unsafe {
            if let Some(pos) = (*p_all_elm).break_ret.iter().position(|&b| b.addr == addr) {
                let b = (*p_all_elm).break_ret.remove(pos);
                memory::breakpoint::restore_byte_of_brkpt(h_proc, b.addr + BASE_ADDR, b.origin_b);
                print_lg(LevelPrint::WarningO, "if a breakpoint is already set on the return address, you will need to remove it manually (it's a b-va breakpoint)".to_string());
            } else {
                print_lg(LevelPrint::ErrorO, format!("no b-ret was set for this addr: {:#x}", addr));
            }
        },
        "watchpoint" | "watch" | "w" => {
            unsafe {
                if let Some(pos) = (*p_all_elm).watchpts.iter().position(|w| w.real_addr64(*ctx) == addr) {
                    watchpoint::clear_dreg(ctx, pos);
                    (*p_all_elm).watchpts.remove(pos);
                    print_lg(LevelPrint::DebugO, "watchpoint has been deleted successfully".to_string());
                } else {
                    print_lg(LevelPrint::ErrorO, format!("the watchpoint for address {:#x} is not found", addr));
                }
            }
        },
        "skip" => unsafe {
            if let Some(pos) = (*p_all_elm).skip_addr.iter().position(|&a| a.addr == addr) {
                let b = (*p_all_elm).skip_addr.remove(pos);
                memory::breakpoint::restore_byte_of_brkpt(h_proc, b.addr + BASE_ADDR, b.origin_b);
            } else {
                print_lg(LevelPrint::ErrorO, format!("no skip has been defined for this function: {target}"));
            }
        },
        "hook" => unsafe {
            if let Some(pos) = (*p_all_elm).hook.iter().position(|h| h.target == addr) {
                let orig_b = (*p_all_elm).hook.remove(pos).origin_byte;
                memory::breakpoint::restore_byte_of_brkpt(h_proc, addr + BASE_ADDR, orig_b);
            } else {
                print_lg(LevelPrint::ErrorO, format!("invalid target: {target}"));
            }
        },
        "def" => unsafe {
            if linev.len() != 4 {
                print_lg(LevelPrint::ErrorO, "Please specify a target for remove".to_string());
                return;
            }
            let target_name = linev[3].to_string();
            match target {
                "func" | "function" => (*p_all_elm).crt_func.retain(|f| f.name != target_name),
                "struct" => (*p_all_elm).struct_def.retain(|s| s.get_name_of_struct() != target_name),
                _ => {
                    print_lg(LevelPrint::ErrorO, format!("unknown element '{target}'"));
                    return;
                }
            }
            print_lg(LevelPrint::DebugO, format!("{target_name} was retained successfully"));
            return;
        },
        _ => {}
    }
}
