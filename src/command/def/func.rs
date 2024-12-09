use crate::pefile::{NtHeaders, NT_HEADER};
use crate::symbol::SYMBOLS_V;
use crate::usage::USAGE_DEF_FUNC;
use crate::ALL_ELM;
use keystone_engine::{Arch, Keystone, KeystoneOutput, Mode, OptionType, OptionValue};
use std::io::{BufRead, BufReader, Write};
use std::{io, ptr};
use std::fs::File;
use winapi::shared::minwindef::LPVOID;
use winapi::um::memoryapi::{VirtualAllocEx, WriteProcessMemory};
use winapi::um::winnt::{HANDLE, MEM_COMMIT, MEM_RESERVE, PAGE_EXECUTE_READWRITE};
use crate::ut::fmt::{print_lg, LevelPrint};

#[derive(Default, Debug)]
pub struct CrtFunc {
    pub code_str: Vec<String>,
    pub name: String,
    pub addr: u64,
}


fn get_keystone_out(engine: *const Keystone, line: &str, addr: u64) -> Result<KeystoneOutput, String> {
    unsafe {
        match (*engine).asm(line.to_string(), addr) {
            Ok(code) => Ok(code),
            Err(e) => Err(format!("{e} : {line}")),
        }
    }
}


impl CrtFunc {
    pub fn get_inline_code(&self) -> String {
        self.code_str.join("; ")
    }

    pub unsafe fn write_cr_func(&mut self, h_proc: HANDLE) -> Result<(), String>{
        let mod_asm = match NT_HEADER.unwrap() {
            NtHeaders::Headers32(_) => Mode::MODE_32,
            NtHeaders::Headers64(_) => Mode::MODE_64,
        };
        let engine = Keystone::new(Arch::X86, mod_asm).unwrap();
        engine.option(OptionType::SYNTAX, OptionValue::SYNTAX_NASM).unwrap();

        let inline_code = self.get_inline_code();
        let result = get_keystone_out(ptr::addr_of!(engine), &inline_code, 0)?.size as usize;

        let addr = VirtualAllocEx(h_proc, 0 as LPVOID, result, MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE);
        if addr.is_null() {
            return Err(format!("Failed to allocate memory: {}", io::Error::last_os_error()));
        }

        let real_res = get_keystone_out(ptr::addr_of!(engine), &inline_code, addr as u64)?.bytes;
        if WriteProcessMemory(h_proc, addr, real_res.as_ptr() as LPVOID, real_res.len(), &mut 0) == 0 {
            return Err(format!("Failed to write func byte at address {:#x} {}", addr as u64, io::Error::last_os_error()));
        }
        self.addr = addr as u64;

        print_lg(LevelPrint::DebugO, format!("the function {} was created successfully at address {:#x}", self.name, self.addr));
        Ok(())
    }
}

fn verify_insn(engine: *const Keystone, asm_line: &str, linev: &Vec<String>) -> Result<(), String> {
    let mut asm_line = asm_line.to_string();
    if let Some(sym) = unsafe { (*ptr::addr_of!(SYMBOLS_V)).symbol_file.iter().find(|s| asm_line.contains(&s.name) && s.name.len() > 3) } {
        asm_line = asm_line.replace(&sym.name, "0x124");
    }
    let asm_line = format!("{}; {asm_line}", linev.join("; "));
    unsafe {
        match (*engine).asm(asm_line, 0) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }
}

fn user_input(engine: *const Keystone, vec_line: &mut Vec<String>) {
    loop {
        print!("    ");
        let mut input = String::new();
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input == ":q" {
            print!("\x1B[1A\x1B[2K");
            io::stdout().flush().unwrap();
            break;
        }

        for line in input.lines() {
            if let Err(e) = push_line(engine, line, vec_line) {
                print_lg(LevelPrint::ErrorO, e);
            }
        }
    }
}


fn push_line(engine: *const Keystone, line: &str, vec_line: &mut Vec<String>) -> Result<(), String> {
    match verify_insn(engine, line, vec_line) {
        Ok(()) => {
            vec_line.push(line.to_string());
            Ok(())
        }
        Err(e) => {
            print!("\x1B[1A\x1B[2K");
            io::stdout().flush().unwrap();
            Err(format!("    {line} ; {e}"))
        }
    }
}

fn file_inpt(engine: *const Keystone, file_path: &str, vec_line: &mut Vec<String>) {
    match File::open(file_path) {
        Ok(file) => {
            let reader = BufReader::new(file);
            for line in reader.lines() {
                match line {
                    Ok(line) => {
                        if line.to_lowercase().contains("section ") {
                            continue;
                        }
                        if let Err(e) = push_line(engine, &line, vec_line) {
                            print_lg(LevelPrint::ErrorO, e);
                            return;
                        }
                    }
                    Err(e) => {
                        print_lg(LevelPrint::ErrorO, e);
                    }
                }
            }
        }
        Err(e) => print_lg(LevelPrint::ErrorO, format!("Failed to open file: {e}")),
    }
}


pub fn crt_func(arg: &[&str], line: &str) {
    if arg.len() < 2 {
        println!("{USAGE_DEF_FUNC}");
        return;
    }

    let mod_asm = {
        if let Some(nt_h) = unsafe { NT_HEADER } {
            match nt_h {
                NtHeaders::Headers32(_) => Mode::MODE_32,
                NtHeaders::Headers64(_) => Mode::MODE_64,
            }
        } else {
            print_lg(LevelPrint::WarningO, "Defaulting to x64 architecture. Use 32-bit code explicitly if needed.");
            Mode::MODE_64
        }
    };

    let engine = Keystone::new(Arch::X86, mod_asm).unwrap();
    engine.option(OptionType::SYNTAX, OptionValue::SYNTAX_INTEL).unwrap();

    let mut vec_line = Vec::new();
    let name_func;
    if  arg.len() > 3 && arg[2] == "file" {
        name_func = arg[2].trim().to_string();
        let tot_len = arg[0].len() + arg[1].len() + name_func.len() + 2;
        let file_path = line[tot_len..].trim().replace("\"", "");
        file_inpt(ptr::addr_of!(engine), &file_path, &mut vec_line);
    } else {
        name_func = arg[1].to_string();
        println!("{}:", name_func);
        user_input(ptr::addr_of!(engine), &mut vec_line);
    }
    unsafe {
        (*ptr::addr_of_mut!(ALL_ELM)).crt_func.push(CrtFunc {
            name: name_func,
            code_str: vec_line,
            addr: 0,
        });
    }
}