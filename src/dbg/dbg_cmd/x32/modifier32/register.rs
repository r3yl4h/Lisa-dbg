use winapi::um::winnt::WOW64_CONTEXT;
use crate::usage;
use crate::ut::cast::str_to;
use crate::ut::fmt::{print_lg, LevelPrint};

pub fn set_register32(linev: &[&str], ctx: &mut WOW64_CONTEXT) {
    if linev.len() < 2 {
        eprintln!("{}", usage::USAGE_SET_REG);
        return;
    }
    let target = linev[0];
    let value_str = linev[1];
    let value = match str_to::<u32>(value_str) {
        Ok(val) => val,
        Err(e) => {
            print_lg(LevelPrint::ErrorO, format!("error to parse '{value_str}' : {e}"));
            return;
        }
    };

    match target {
        "eax" => set_reg32(&mut ctx.Eax, value),
        "ebx" => set_reg32(&mut ctx.Ebx, value),
        "ecx" => set_reg32(&mut ctx.Ecx, value),
        "edx" => set_reg32(&mut ctx.Edx, value),
        "esi" => set_reg32(&mut ctx.Esi, value),
        "edi" => set_reg32(&mut ctx.Edi, value),
        "esp" => set_reg32(&mut ctx.Esp, value),
        "ebp" => set_reg32(&mut ctx.Ebp, value),
        "eip" => set_reg32(&mut ctx.Eip, value),
        "flag" | "eflag" => set_reg32(&mut ctx.EFlags, value),
        "cs" => set_reg32(&mut ctx.SegCs, value),
        "ds" => set_reg32(&mut ctx.SegDs, value),
        "es" => set_reg32(&mut ctx.SegEs, value),
        "fs" => set_reg32(&mut ctx.SegFs, value),
        "gs" => set_reg32(&mut ctx.SegGs, value),
        "ss" => set_reg32(&mut ctx.SegSs, value),
        
        "ax" => set_reg16(&mut ctx.Eax, value as u16),
        "bx" => set_reg16(&mut ctx.Ebx, value as u16),
        "cx" => set_reg16(&mut ctx.Ecx, value as u16),
        "dx" => set_reg16(&mut ctx.Edx, value as u16),
        
        "al" => set_reg8(&mut ctx.Eax, value as u8, true),
        "bl" => set_reg8(&mut ctx.Ebx, value as u8, true),
        "cl" => set_reg8(&mut ctx.Ecx, value as u8, true),
        "dl" => set_reg8(&mut ctx.Edx, value as u8, true),
        
        "ah" => set_reg8(&mut ctx.Eax, value as u8, false),
        "bh" => set_reg8(&mut ctx.Ebx, value as u8, false),
        "ch" => set_reg8(&mut ctx.Ecx, value as u8, false),
        "dh" => set_reg8(&mut ctx.Edx, value as u8, false),

        _ => {
            print_lg(LevelPrint::ErrorO, format!("Unknown register: {target}"));
            return;
        }
    }
}

fn set_reg32(reg: &mut u32, value: u32) {
    *reg = value;
}

fn set_reg16(reg: &mut u32, value: u16) {
    *reg = (*reg & 0xFFFF0000) | (value as u32);
}

fn set_reg8(reg: &mut u32, value: u8, low: bool) {
    if low {
        *reg = (*reg & 0xFFFFFF00) | (value as u32);
    } else {
        *reg = (*reg & 0xFFFF00FF) | ((value as u32) << 8);
    }
}
