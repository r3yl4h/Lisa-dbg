use crate::command::def::func::CrtFunc;
use crate::dbg::BASE_ADDR;
use crate::pefile::{NtHeaders, NT_HEADER};
use crate::symbol::SYMBOLS_V;
use keystone_engine::{Arch, Keystone, Mode, OptionType, OptionValue};
use std::{io, ptr};
use winapi::shared::minwindef::LPVOID;
use winapi::shared::ntdef::HANDLE;
use winapi::um::memoryapi::{VirtualAllocEx, WriteProcessMemory};
use winapi::um::winnt::{MEM_COMMIT, MEM_RESERVE, PAGE_EXECUTE_READWRITE};
use crate::ut::fmt::{print_lg, LevelPrint};

fn get_all_size(engine: *const Keystone, linev: &[String]) -> Result<usize, String> {
    let mut result = 0;
    for line in linev {
        let mut line = line.to_string();
        match get_real_insn(&mut line) {
            Ok(()) => unsafe {
                match (*engine).asm(line.to_string(), result as u64) {
                    Ok(code) => result += code.size as usize,
                    Err(e) => return Err(format!("{e} : {line}")),
                }
            },
            Err(e) => return Err(e),
        }
    }
    Ok(result)
}

fn get_real_insn(line: &mut String) -> Result<(), String> {
    unsafe {
        if let Some(sym) = (*ptr::addr_of!(SYMBOLS_V)).symbol_file.iter().find(|s| line.contains(&s.name)) {
            if sym.name.len() > 3 {
                if sym.offset > 0 {
                    *line = line.replace(&sym.name, &format!("{:#x}", sym.offset as u64 + BASE_ADDR));
                } else {
                    return Err(format!("you can only specify functions or global variables: {}", sym.name));
                }
            }
        }
    }
    Ok(())
}

pub unsafe fn set_cr_function(h_proc: HANDLE, crt_func: &mut CrtFunc) -> Result<(), String>{
    let mod_asm = match NT_HEADER.unwrap() {
        NtHeaders::Headers32(_) => Mode::MODE_32,
        NtHeaders::Headers64(_) => Mode::MODE_64,
    };
    let engine = Keystone::new(Arch::X86, mod_asm).unwrap();
    engine.option(OptionType::SYNTAX, OptionValue::SYNTAX_NASM).unwrap();

    let result = match get_all_size(ptr::addr_of!(engine), &crt_func.code_str) {
        Ok(size) => size,
        Err(e) => {
            return Err(e)
        }
    };

    let addr = VirtualAllocEx(h_proc, 0 as LPVOID, result, MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE);
    if addr.is_null() {
        return Err(format!("Failed to allocate memory: {}", io::Error::last_os_error()));
    }

    let mut ip = addr as u64;
    crt_func.addr = ip;
    for line in &crt_func.code_str {
        let mut line = line.to_string();
        match get_real_insn(&mut line) {
            Ok(()) => match engine.asm(line.clone(), ip) {
                Ok(code) => {
                    if WriteProcessMemory(h_proc, ip as LPVOID, code.bytes.as_ptr() as LPVOID, code.size as usize, &mut 0) == 0 {
                        return Err(format!("Failed to write memory at address {:#x} : {}", ip, io::Error::last_os_error()));
                    }
                    ip += code.size as u64;
                }
                Err(e) => print_lg(LevelPrint::ErrorO, format!("{e}: {line}")),
            },
            Err(e) => print_lg(LevelPrint::ErrorO, e),
        }
    }
    print_lg(LevelPrint::DebugO, format!("the function {} was created successfully at address {:#x}", crt_func.name, crt_func.addr));
    Ok(())
}
