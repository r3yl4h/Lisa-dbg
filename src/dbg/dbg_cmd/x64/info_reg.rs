use crate::ut::fmt::{print_lg, LevelPrint, RESET_COLOR, VALID_COLOR};
use crate::ut::fmt::VALUE_COLOR;
use crate::usage;
use winapi::um::winnt::{CONTEXT, M128A};
use crate::dbg::memory::deref_mem;

pub enum Value {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(M128A),
    Un,
}

pub trait ToValue {
    fn str_to_value_ctx(self, target: &str) -> Value;
}

impl ToValue for CONTEXT {
    fn str_to_value_ctx(self, target: &str) -> Value {
        let target = target.to_lowercase();
        unsafe {
            match target.as_str() {
                "rax" => Value::U64(self.Rax),
                "rbx" => Value::U64(self.Rbx),
                "rcx" => Value::U64(self.Rcx),
                "rdx" => Value::U64(self.Rdx),
                "rsi" => Value::U64(self.Rsi),
                "rdi" => Value::U64(self.Rdi),
                "rbp" => Value::U64(self.Rbp),
                "rsp" => Value::U64(self.Rsp),
                "rip" => Value::U64(self.Rip),
                "r8" => Value::U64(self.R8),
                "r9" => Value::U64(self.R9),
                "r10" => Value::U64(self.R10),
                "r11" => Value::U64(self.R11),
                "r12" => Value::U64(self.R12),
                "r13" => Value::U64(self.R13),
                "r14" => Value::U64(self.R14),
                "r15" => Value::U64(self.R15),
                
                "eax" => Value::U32((self.Rax & 0xFFFFFFFF) as u32),
                "ebx" => Value::U32((self.Rbx & 0xFFFFFFFF) as u32),
                "ecx" => Value::U32((self.Rcx & 0xFFFFFFFF) as u32),
                "edx" => Value::U32((self.Rdx & 0xFFFFFFFF) as u32),
                "esi" => Value::U32((self.Rsi & 0xFFFFFFFF) as u32),
                "edi" => Value::U32((self.Rdi & 0xFFFFFFFF) as u32),
                "ebp" => Value::U32((self.Rbp & 0xFFFFFFFF) as u32),
                "esp" => Value::U32((self.Rsp & 0xFFFFFFFF) as u32),
                
                "ax" => Value::U16((self.Rax & 0xFFFF) as u16),
                "bx" => Value::U16((self.Rbx & 0xFFFF) as u16),
                "cx" => Value::U16((self.Rcx & 0xFFFF) as u16),
                "dx" => Value::U16((self.Rdx & 0xFFFF) as u16),
                "si" => Value::U16((self.Rsi & 0xFFFF) as u16),
                "di" => Value::U16((self.Rdi & 0xFFFF) as u16),
                "bp" => Value::U16((self.Rbp & 0xFFFF) as u16),
                "sp" => Value::U16((self.Rsp & 0xFFFF) as u16),

      
                "al" => Value::U8((self.Rax & 0xFF) as u8),
                "bl" => Value::U8((self.Rbx & 0xFF) as u8),
                "cl" => Value::U8((self.Rcx & 0xFF) as u8),
                "dl" => Value::U8((self.Rdx & 0xFF) as u8),
                
                "ah" => Value::U8(((self.Rax >> 8) & 0xFF) as u8),
                "bh" => Value::U8(((self.Rbx >> 8) & 0xFF) as u8),
                "ch" => Value::U8(((self.Rcx >> 8) & 0xFF) as u8),
                "dh" => Value::U8(((self.Rdx >> 8) & 0xFF) as u8),
                
                "segcs" | "cs" => Value::U16(self.SegCs as u16),
                "segds" | "ds" => Value::U16(self.SegDs as u16),
                "seges" | "es" => Value::U16(self.SegEs as u16),
                "segfs" | "fs" => Value::U16(self.SegFs as u16),
                "seggs" | "gs" => Value::U16(self.SegGs as u16),
                "segss" | "ss" => Value::U16(self.SegSs as u16),
                
                "eflags" | "flags" => Value::U64(self.EFlags as u64),
                
                "xmm0" => Value::U128(self.u.s().Xmm0),
                "xmm1" => Value::U128(self.u.s().Xmm1),
                "xmm2" => Value::U128(self.u.s().Xmm2),
                "xmm3" => Value::U128(self.u.s().Xmm3),
                "xmm4" => Value::U128(self.u.s().Xmm4),
                "xmm5" => Value::U128(self.u.s().Xmm5),
                "xmm6" => Value::U128(self.u.s().Xmm6),
                "xmm7" => Value::U128(self.u.s().Xmm7),
                "xmm8" => Value::U128(self.u.s().Xmm8),
                "xmm9" => Value::U128(self.u.s().Xmm9),
                "xmm10" => Value::U128(self.u.s().Xmm10),
                "xmm11" => Value::U128(self.u.s().Xmm11),
                "xmm12" => Value::U128(self.u.s().Xmm12),
                "xmm13" => Value::U128(self.u.s().Xmm13),
                "xmm14" => Value::U128(self.u.s().Xmm14),
                "xmm15" => Value::U128(self.u.s().Xmm15),
                
                "mxcsr" => Value::U32(self.MxCsr),
                
                _ => Value::Un,
            }
        }
    }
}

pub const ALL_REG64: [&str; 48] = [
    "rax", "rbx", "rcx", "rdx", "rsi", "rdi", "rbp", "rsp", "rip", "r8", "r9", "r10", "r11", "r12",
    "r13", "r14", "r15", "segcs", "segds", "seges", "segfs", "seggs", "segss", "eflags", "cs",
    "ds", "es", "fs", "gs", "ss", "flags", "xmm0", "xmm1", "xmm2", "xmm3", "xmm4", "xmm5", "xmm6",
    "xmm7", "xmm8", "xmm9", "xmm10", "xmm11", "xmm12", "xmm13", "xmm14", "xmm15", "mxcsr",
];

pub unsafe fn handle_reg(linev: &[&str], ctx: CONTEXT) {
    match linev.get(1) {
        Some(&"all-reg") | Some(&"all-register") => {
            for reg_name in ALL_REG64.iter().filter(|&reg| reg.starts_with("r")) {
                let value = ctx.str_to_value_ctx(reg_name);
                match value {
                    Value::U64(v) => {
                        let str1 = deref_mem::espc(&v.to_le_bytes());
                        println!("{:<4} = {VALUE_COLOR}{:>#18x}{RESET_COLOR} | {VALUE_COLOR}{:>20}{RESET_COLOR} | {:>4} \"{str1}\"", reg_name, v, v as i64, " ")
                    },
                    _ => {}
                }
            }
        }

        Some(&"all-seg") | Some(&"all-segment") => {
            for reg_name in ALL_REG64.iter().filter(|&reg| reg.ends_with("s") && reg.len() == 2) {
                let value = ctx.str_to_value_ctx(reg_name);
                match value {
                    Value::U16(v) => 
                        println!("{:<3} = {VALUE_COLOR}{:>#18x}{RESET_COLOR}", reg_name, v),
                    
                    _ => {}
                }
            }
        }

        Some(&"all-vec") | Some(&"all-vector") => {
            for reg_name in ALL_REG64.iter().filter(|r| r.starts_with("xmm")) {
                let value = ctx.str_to_value_ctx(reg_name);
                match value {
                    Value::U128(v) => {
                        let str1 = deref_mem::espc(&v.Low.to_le_bytes());
                        let str2 = deref_mem::espc(&v.High.to_le_bytes());
                        println!("{:<6} = {{\"{str1}{str2}\"}}", reg_name);
                        println!("{:<6} = {{{}{:#x}, {:#x}{}}}", "_m128i", VALUE_COLOR, v.Low, v.High, RESET_COLOR);
                        let float: [f32; 4] = unsafe { std::mem::transmute(v) };
                        println!("{:<6} = {{{}{}, {}, {}, {}{}}}", "_m128", VALUE_COLOR, float[0], float[1], float[2], float[3], RESET_COLOR);
                        let double: [f64;2] = unsafe { std::mem::transmute(v) };
                        println!("{:<6} = {{{VALUE_COLOR}{}, {}{RESET_COLOR}}}", "_m128d", double[0], double[1]);
                        println!();
                    },
                    _ => {}
                }
            }
        }

        Some(&"all") => {
            for reg_name in ALL_REG64 {
                let reg_value = ctx.str_to_value_ctx(reg_name);
                match reg_value {
                    Value::U64(v) => {
                        let str_s = deref_mem::espc(&v.to_le_bytes());
                        println!("{:<6} = {VALUE_COLOR}{:>#18x}{RESET_COLOR} | {VALUE_COLOR}{:>20}{RESET_COLOR} | \"{str_s}\"", reg_name, v, v as i64);
                    }
                    Value::U32(v) => {
                        let str1 = deref_mem::espc(&v.to_le_bytes());
                        println!("{:<5} = {VALUE_COLOR}{:#x}{RESET_COLOR} | {VALUE_COLOR}{}{RESET_COLOR} | \"{str1}\"", reg_name, v, v as i32);
                    }
                    Value::U16(v) => {
                        let str1 = deref_mem::espc(&v.to_le_bytes());
                        println!("{:<5} = {VALUE_COLOR}{:#x}{RESET_COLOR} | {VALUE_COLOR}{}{RESET_COLOR} | \"{str1}\"", reg_name, v, v as i16);
                    }
                    Value::U8(v) => {
                        println!("{:<5} = {VALUE_COLOR}{:#x}{RESET_COLOR} | {VALUE_COLOR}{}{RESET_COLOR} | '{VALID_COLOR}{}{RESET_COLOR}'", reg_name, v, v as i8, v as char);
                    }
                    Value::Un => print_lg(LevelPrint::ErrorO, format!("unknow register : '{reg_name}'")),
                    _ => {}
                }
            }
        }

        Some(register) => {
            let reg_value = ctx.str_to_value_ctx(register);
            match reg_value {
                Value::U128(v) => {
                    let str1 = deref_mem::espc(&v.Low.to_le_bytes());
                    let str2 = deref_mem::espc(&v.High.to_le_bytes());
                    println!("{:<6} = {{\"{str1}{str2}\"}}", register);
                    println!("{:<6} = {{{}{:#x}, {:#x}{}}}", "_m128i", VALUE_COLOR, v.Low, v.High, RESET_COLOR);
                    let float: [f32; 4] = unsafe { std::mem::transmute(v) };
                    println!("{:<6} = {{{}{}, {}, {}, {}{}}}", "_m128", VALUE_COLOR, float[0], float[1], float[2], float[3], RESET_COLOR);
                    let double: [f64;2] = unsafe { std::mem::transmute(v) };
                    println!("{:<6} = {{{VALUE_COLOR}{}, {}{RESET_COLOR}}}", "_m128d", double[0], double[1]);
                    println!();
                }
                Value::U64(v) => {
                    let str1 = deref_mem::espc(&v.to_le_bytes());
                    println!("{:<5} = {VALUE_COLOR}{:#x}{RESET_COLOR} | {VALUE_COLOR}{}{RESET_COLOR} | \"{str1}\"", register, v, v as i64);
                }
                Value::U32(v) => {
                    let str1 = deref_mem::espc(&v.to_le_bytes());
                    println!("{:<5} = {VALUE_COLOR}{:#x}{RESET_COLOR} | {VALUE_COLOR}{}{RESET_COLOR} | \"{str1}\"", register, v, v as i32);
                }
                Value::U16(v) => {
                    let str1 = deref_mem::espc(&v.to_le_bytes());
                    println!("{:<5} = {VALUE_COLOR}{:#x}{RESET_COLOR} | {VALUE_COLOR}{}{RESET_COLOR} | \"{str1}\"", register, v, v as i16);
                }
                Value::U8(v) => {
                    println!("{:<5} = {VALUE_COLOR}{:#x}{RESET_COLOR} | {VALUE_COLOR}{}{RESET_COLOR} | '{VALID_COLOR}{}{RESET_COLOR}'", register, v, v as i8, v as char);
                }
                Value::Un => print_lg(LevelPrint::ErrorO, format!("unknow register : '{register}'")),
            }
        }
        None => println!("{}", usage::USAGE_REG),
    }
}
