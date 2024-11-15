use crate::ut::fmt::*;
use crate::usage;
use winapi::um::winnt::WOW64_CONTEXT;
use crate::dbg::memory::deref_mem;

pub const ALL_REG32: [&str; 34] = [
    "eax",
    "ebx",
    "ecx",
    "edx",
    "esi",
    "edi",
    "ebp",
    "esp",
    "eip",
    "cs",
    "segcs",
    "ds",
    "segds",
    "es",
    "seges",
    "fs",
    "segfs",
    "gs",
    "seggs",
    "ss",
    "segss",
    "flag",
    "eflag",
    "ctrl-word",
    "control-word",
    "status-word",
    "tag-word",
    "err-offset",
    "error-offset",
    "err-select",
    "error-selector",
    "data-offset",
    "data-selector",
    "data-select",
];

pub trait ToValue32 {
    fn str_to_ctx(self, target: &str) -> u32;
}

impl ToValue32 for WOW64_CONTEXT {
    fn str_to_ctx(self, target: &str) -> u32 {
        let target = target.to_lowercase();
        match target.as_str() {
            "eax" => self.Eax,
            "ebx" => self.Ebx,
            "ecx" => self.Ecx,
            "edx" => self.Edx,
            "esi" => self.Esi,
            "edi" => self.Edi,
            "ebp" => self.Ebp,
            "esp" => self.Esp,
            "eip" => self.Eip,
            
            "ax" => self.Eax & 0xFFFF,
            "bx" => self.Ebx & 0xFFFF,
            "cx" => self.Ecx & 0xFFFF,
            "dx" => self.Edx & 0xFFFF,
            
            "al" => self.Eax & 0xFF,
            "bl" => self.Ebx & 0xFF,
            "cl" => self.Ecx & 0xFF,
            "dl" => self.Edx & 0xFF,
            
            "ah" => (self.Eax >> 8) & 0xFF,
            "bh" => (self.Ebx >> 8) & 0xFF,
            "ch" => (self.Ecx >> 8) & 0xFF,
            "dh" => (self.Edx >> 8) & 0xFF,
            
            "cs" | "segcs" => self.SegCs,
            "ds" | "segds" => self.SegDs,
            "es" | "seges" => self.SegEs,
            "fs" | "segfs" => self.SegFs,
            
            "ctrl-word" | "control-word" => self.FloatSave.ControlWord,
            "status-word" => self.FloatSave.StatusWord,
            "tag-word" => self.FloatSave.TagWord,
            "err-offset" | "error-offset" => self.FloatSave.ErrorOffset,
            "err-select" | "error-select" => self.FloatSave.ErrorSelector,
            "data-offset" => self.FloatSave.DataOffset,
            "data-select" => self.FloatSave.DataSelector,
            _ => 0,
        }
    }
}

pub unsafe fn handle_reg(linev: &[&str], ctx: WOW64_CONTEXT) {
    match linev.get(1) {
        Some(&"all-reg") | Some(&"all-register") => {
            for reg_name in ALL_REG32.iter().filter(|&&s| s.len() != 3) {
                let value = ctx.str_to_ctx(reg_name);
                let str1 = deref_mem::espc(&value.to_le_bytes());
                println!("{:<3}: {VALUE_COLOR}{:>#14x}{RESET_COLOR} | {VALUE_COLOR}{:>20}{RESET_COLOR} | {:<4} {str1}", reg_name, value, value as i32, " ");
            }
        }

        Some(&"all-seg") | Some(&"all-segment") => {
            for reg_name in ALL_REG32.iter().filter(|&reg| reg.ends_with('s') && reg.len() == 2) {
                let value = ctx.str_to_ctx(reg_name);
                let str1 = deref_mem::espc(&value.to_le_bytes());
                println!("{:<3}: {VALUE_COLOR}{:>#14x}{RESET_COLOR} | {VALUE_COLOR}{:>20}{RESET_COLOR} | {:<4} {str1}", reg_name, value, value as i32, " ");
            }
        }

        Some(&"all") => {
            for reg_name in ALL_REG32 {
                let reg_value = ctx.str_to_ctx(reg_name);
                let mut rname = reg_name.to_string();
                rname.truncate(6);
                let str1 = deref_mem::espc(&reg_value.to_le_bytes());
                println!("{:<6}: {VALUE_COLOR}{:>#14x}{RESET_COLOR} | {VALUE_COLOR}{:>20}{RESET_COLOR} | {:<4} {str1}", rname, reg_value, reg_value as i32, " ");
            }
        }

        Some(register) => {
            if let Some(reg_name) = ALL_REG32.iter().find(|&r| r == register) {
                let reg_value = ctx.str_to_ctx(reg_name);
                let str1 = deref_mem::espc(&reg_value.to_le_bytes());
                println!("{:<6}: {VALUE_COLOR}{:>#14x}{RESET_COLOR} | {VALUE_COLOR}{:>20}{RESET_COLOR} | {:<4} {str1}", reg_name, reg_value, reg_value as i32, " ");
            } else {
                print_lg(LevelPrint::ErrorO, format!("Unknown register: {register}"));
            }
        }
        None => {
            println!("{}", usage::USAGE_INFO);
        }
    }
}
