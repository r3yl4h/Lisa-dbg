use crate::dbg::dbg_cmd::x64::info_reg::Value;
use crate::usage;

use winapi::um::winnt::{CONTEXT, M128A};
use crate::ut::cast::str_to;
use crate::ut::fmt::{print_lg, LevelPrint};

pub fn set_register64(linev: &[&str], ctx: &mut CONTEXT) {
    if linev.len() < 2 {
        eprintln!("{}", usage::USAGE_SET_REG);
        return;
    }
    let target = linev[0];
    let value_str = linev[1];
    let value = match str_to::<u64>(value_str) {
        Ok(val) => Value::U64(val),
        Err(_) => match str_to::<u128>(value_str) {
            Ok(val) => {
                let value = M128A {
                    Low: (val & 0xFFFFFFFFFFFFFFFF) as u64,
                    High: (val >> 64) as i64,
                };
                Value::U128(value)
            }
            Err(e) => {
                print_lg(LevelPrint::ErrorO, format!("error to parse '{value_str}' : {e}"));
                return;
            }
        },
    };

    match target {
        "rax" => set_reg64(&mut ctx.Rax, value),
        "rbx" => set_reg64(&mut ctx.Rbx, value),
        "rcx" => set_reg64(&mut ctx.Rcx, value),
        "rdx" => set_reg64(&mut ctx.Rdx, value),
        "rsi" => set_reg64(&mut ctx.Rsi, value),
        "rdi" => set_reg64(&mut ctx.Rdi, value),
        "rsp" => set_reg64(&mut ctx.Rsp, value),
        "rbp" => set_reg64(&mut ctx.Rbp, value),
        "rip" => set_reg64(&mut ctx.Rip, value),
        "r8" => set_reg64(&mut ctx.R8, value),
        "r9" => set_reg64(&mut ctx.R9, value),
        "r10" => set_reg64(&mut ctx.R10, value),
        "r11" => set_reg64(&mut ctx.R11, value),
        "r12" => set_reg64(&mut ctx.R12, value),
        "r13" => set_reg64(&mut ctx.R13, value),
        "r14" => set_reg64(&mut ctx.R14, value),
        "r15" => set_reg64(&mut ctx.R15, value),

   
        "eax" => set_reg32(&mut ctx.Rax, value),
        "ebx" => set_reg32(&mut ctx.Rbx, value),
        "ecx" => set_reg32(&mut ctx.Rcx, value),
        "edx" => set_reg32(&mut ctx.Rdx, value),
        "esi" => set_reg32(&mut ctx.Rsi, value),
        "edi" => set_reg32(&mut ctx.Rdi, value),
        "ebp" => set_reg32(&mut ctx.Rbp, value),
        "esp" => set_reg32(&mut ctx.Rsp, value),

      
        "ax" => set_reg16(&mut ctx.Rax, value),
        "bx" => set_reg16(&mut ctx.Rbx, value),
        "cx" => set_reg16(&mut ctx.Rcx, value),
        "dx" => set_reg16(&mut ctx.Rdx, value),
        "si" => set_reg16(&mut ctx.Rsi, value),
        "di" => set_reg16(&mut ctx.Rdi, value),
        "bp" => set_reg16(&mut ctx.Rbp, value),
        "sp" => set_reg16(&mut ctx.Rsp, value),

   
        "al" => set_reg8(&mut ctx.Rax, value),
        "bl" => set_reg8(&mut ctx.Rbx, value),
        "cl" => set_reg8(&mut ctx.Rcx, value),
        "dl" => set_reg8(&mut ctx.Rdx, value),

     
        "xmm0" => set_reg_simd(unsafe { &mut ctx.u.s_mut().Xmm0 }, value),
        "flag" => set_flag(&mut ctx.EFlags, value),

        _ => {
            print_lg(LevelPrint::ErrorO, format!("Unknown register: {target}"));
            return;
        }
    }
}

fn set_flag(eflag: &mut u32, value: Value) {
    match value {
        Value::U64(value) => {
            if (u32::MAX as u64) < value {
                print_lg(LevelPrint::ErrorO, "you cannot put a value above 32bits in this destination")
            }
            *eflag = value as u32
        }
        Value::U128(_) => print_lg(LevelPrint::ErrorO, "you cannot put a value above 32bits in this destination"),
        _ => print_lg(LevelPrint::ErrorO, "unknown register"),
    }
}

fn set_reg_simd(reg: &mut M128A, value: Value) {
    match value {
        Value::U64(value) => {
            reg.Low = value;
            reg.High = 0;
        }
        Value::U128(value) => *reg = value,
        _ => {}
    }
}

fn set_reg64(reg: &mut u64, value: Value) {
    match value {
        Value::U64(val) => *reg = val,
        Value::U128(_) => print_lg(LevelPrint::ErrorO, "you can't put a 128bit value into a 64bit register"),
        _ => {}
    }
}

fn set_reg32(reg: &mut u64, value: Value) {
    match value {
        Value::U64(val) => *reg = (*reg & 0xFFFFFFFF00000000) | (val & 0xFFFFFFFF),
        _ => print_lg(LevelPrint::ErrorO, "Invalid value for 32bit register"),
    }
}

fn set_reg16(reg: &mut u64, value: Value) {
    match value {
        Value::U64(val) => *reg = (*reg & 0xFFFFFFFFFFFF0000) | (val & 0xFFFF),
        _ => print_lg(LevelPrint::ErrorO, "Invalid value for 16bit register"),
    }
}

fn set_reg8(reg: &mut u64, value: Value) {
    match value {
        Value::U64(val) => *reg = (*reg & 0xFFFFFFFFFFFFFF00) | (val & 0xFF),
        _ => print_lg(LevelPrint::ErrorO, "Invalid value for 8bit register"),
    }
}
