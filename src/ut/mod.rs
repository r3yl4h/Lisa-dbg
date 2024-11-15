use std::ptr;
use winapi::um::winnt::{CONTEXT, WOW64_CONTEXT};
use crate::dbg::dbg_cmd::x32::info_reg::ToValue32;
use crate::dbg::dbg_cmd::x64::info_reg::{ToValue, Value};
use crate::dbg::RealAddr;
use crate::symbol::SYMBOLS_V;
use crate::ut::cast::str_to;
use crate::ut::fmt::*;

pub mod cast;
pub mod fmt;
pub mod mem;

pub fn get_addr_va(addr_str: &str, ctx: CONTEXT) -> Result<u64, String> {
    match str_to::<u64>(addr_str) {
        Ok(addr) => Ok(addr),
        Err(e) => unsafe {
            if e.to_string().contains("invalid digit") {
                let s = &raw const SYMBOLS_V;
                if let Some(sym) = (*s).symbol_file.iter().find(|s| s.name == addr_str) {
                    Ok(sym.real_addr64(ctx))
                } else {
                    match ctx.str_to_value_ctx(addr_str) {
                        Value::U64(addr) => Ok(addr),
                        _ => Err(format!("Invalid target: '{addr_str}'{}", RESET_COLOR)),
                    }
                }
            } else {
                Err(format!("Invalid target: '{addr_str}'{}", RESET_COLOR))
            }
        },
    }
}



pub fn get_addr_va32(addr_str: &str, ctx: WOW64_CONTEXT) -> Result<u32, String> {
    match str_to::<u32>(addr_str) {
        Ok(addr) => Ok(addr),
        Err(e) => unsafe {
            if e.to_string().contains("invalid digit") {
                let s = &raw const SYMBOLS_V;
                if let Some(sym) = (*s).symbol_file.iter().find(|s| s.name == addr_str) {
                    Ok(sym.real_addr32(ctx))
                } else {
                    let ad = ctx.str_to_ctx(addr_str);
                    if ad != 0 {
                        Ok(ad)
                    } else {
                        Err(format!("Invalid target: '{addr_str}'"))
                    }
                }
            } else {
                Err(format!("Invalid target: '{addr_str}'"))
            }
        },
    }
}



pub fn get_addr_br(addr_str: &str) -> Result<u64, String> {
    match str_to::<u64>(addr_str) {
        Ok(value) => Ok(value),
        Err(_) => unsafe {
            let s = ptr::addr_of!(SYMBOLS_V);
            if let Some(sym) = (*s).symbol_file.iter().find(|s| s.name == addr_str) {
                if sym.offset > 0 {
                    Ok(sym.offset as u64)
                } else {
                    Err("the specified symbol cannot have a negative offset".to_string())
                }
            } else {
                Err(format!("invalid target : {addr_str}"))
            }
        },
    }
}







