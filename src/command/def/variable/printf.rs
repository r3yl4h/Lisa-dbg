use std::{io, ptr};
use winapi::shared::minwindef::LPVOID;
use winapi::um::memoryapi::ReadProcessMemory;
use winapi::um::winnt::HANDLE;
use crate::cli::ALL_ELM;
use crate::command::def::variable::Var;
use crate::usage::USAGE_PRINTF_VAR;
use crate::ut::fmt::{print_lg, LevelPrint};

pub fn printf_var(linev: &[&str], line: &str, handle: HANDLE) {
    if linev.len() < 2 {
        print_lg(LevelPrint::DebugO, USAGE_PRINTF_VAR);
        return;
    }

    let mut str_u = String::new();
    let mut ipn = String::new();
    let n_line = &line[linev[0].len()+1..];


    if let Err(e) = get_str(n_line, &mut ipn, &mut str_u) {
        print_lg(LevelPrint::ErrorO, e);
        return;
    }

    if str_u.is_empty() {
        if let Err(e) = get_str_from_var_name(ipn, &mut str_u, handle) {
            print_lg(LevelPrint::ErrorO, format!("the first argument is invalid : {e}"));
            return;
        }
    }

    let mut str_final = String::new();
    let mut chars = str_u.chars().peekable();
    let mut argv = n_line[str_u.len()+2..].split(",").collect::<Vec<&str>>();
    argv.iter_mut().for_each(|f|*f = f.trim());
    let mut i_arg = 0;

    while let Some(c) = chars.next() {
        match c {
            '%' => {
                if let Some(next_char) = chars.peek() {
                    match next_char {
                        'd' | 'i' => {
                            let arg = argv[i_arg];
                            if let Some(var) = Var::get_var_with_name(unsafe { (*&raw const ALL_ELM).var_def.clone() }, arg) {
                                match var.to_i32() {
                                    Ok(value) => str_final.push_str(&value.to_string()),
                                    Err(e) => print_lg(LevelPrint::ErrorO, e),
                                }
                            }
                            i_arg += 1;
                            chars.next();
                        },
                        'u' => {
                            let arg = argv[i_arg];
                            if let Some(var) = Var::get_var_with_name(unsafe { (*&raw const ALL_ELM).var_def.clone() }, arg) {
                                match var.to_u32() {
                                    Ok(value) => str_final.push_str(&value.to_string()),
                                    Err(e) => print_lg(LevelPrint::ErrorO, e),
                                }
                            }
                            i_arg += 1;
                            chars.next();
                        }
                        'x' => {
                            let arg = argv[i_arg];
                            if let Some(var) = Var::get_var_with_name(unsafe { (*&raw const ALL_ELM).var_def.clone() }, arg) {
                                match var.to_u32() {
                                    Ok(value) => str_final.push_str(&format!("{:x}", value)),
                                    Err(e) => print_lg(LevelPrint::ErrorO, e),
                                }
                            }
                            i_arg += 1;
                            chars.next();
                        }
                        'X' => {
                            let arg = argv[i_arg];
                            if let Some(var) = Var::get_var_with_name(unsafe { (*&raw const ALL_ELM).var_def.clone() }, arg) {
                                match var.to_u32() {
                                    Ok(value) => str_final.push_str(&format!("{:X}", value)),
                                    Err(e) => print_lg(LevelPrint::ErrorO, e),
                                }
                            }
                            i_arg += 1;
                            chars.next();
                        }
                        'f' => {
                            let arg = argv[i_arg];
                            if let Some(var) = Var::get_var_with_name(unsafe { (*&raw const ALL_ELM).var_def.clone() }, arg) {
                                match var.to_f32() {
                                    Ok(value) => str_final.push_str(&value.to_string()),
                                    Err(e) => print_lg(LevelPrint::ErrorO, e),
                                }
                            }
                            i_arg += 1;
                            chars.next();
                        }
                        's' => {
                            let arg = argv[i_arg];
                            if arg.starts_with("\"") {
                                let mut str_a = String::new();
                                if let Err(e) = get_str(arg, &mut String::new(), &mut str_a) {
                                    print_lg(LevelPrint::ErrorO, e);
                                }else {
                                    str_final.push_str(&str_a);
                                }
                            }else {
                                let mut str_a = String::new();
                                if let Err(e) = get_str_from_var_name(arg.to_string(), &mut str_a, handle) {
                                    print_lg(LevelPrint::ErrorO, e);
                                }else {
                                    str_final.push_str(&str_a);
                                }
                            }
                            i_arg += 1;
                            chars.next();
                        },
                        _ => str_final.push(c),
                    }
                } else {
                    str_final.push('%');
                }
            },
            _ => str_final.push(c),
        }
    }
    println!("{}", str_final);
}



fn get_str_from_var_name(var_name: String, out: &mut String, handle: HANDLE) -> Result<(), anyhow::Error> {
    if let Some(var) = Var::get_var_with_name(unsafe { (*&raw const ALL_ELM).var_def.clone() }, &var_name) {
        *out = if var.type_p.cout_elm() > 1 {
            String::from_utf8_lossy(&var.value).to_string()
        } else {
            let addr_ptr = match var.to_u64() {
                Ok(v) => v as LPVOID,
                Err(e) => return Err(e),
            };
            match read_str_from_lpvoid(handle, addr_ptr) {
                Ok(v) => v,
                Err(e) =>  return Err(e),
            }
        };
        Ok(())
    }else {
        Err(anyhow::Error::msg(format!("no variable is named '{var_name}'")))
    }
}

fn get_str(n_line: &str, ipn: &mut String, str_u: &mut String) -> Result<(), String>{
    for (i, c) in n_line.chars().enumerate() {
        if c == '"' {
            if let Some(pos) = n_line[i+1..].find("\"") {
                str_u.push_str(&n_line[i+1..pos+1]);
                break;
            } else {
                let mut mt = n_line[i..].to_string();
                mt.truncate(3);
                return Err(format!("please close the quotation marks : {mt}"));
            }
        } else if c == ' ' {
            break;
        } else {
            ipn.push(c);
        }
    }
    Ok(())
}



pub fn read_str_from_lpvoid(handle: HANDLE, addr: LPVOID) -> Result<String, anyhow::Error>{
    let mut addr = addr;
    let mut str_i = String::new();
    loop {
        let mut c = 0u8;
        if unsafe {ReadProcessMemory(handle, addr, ptr::addr_of_mut!(c) as LPVOID, 1, &mut 0)} == 0 {
            return Err(anyhow::Error::msg(format!("failed to read memory at address {:#x} : {}", addr as u64, io::Error::last_os_error())))
        }
        if c == 0 {
            return Ok(str_i)
        }else {
            str_i.push(c as char)
        }
        addr = (addr as u64 + 1) as LPVOID;
    }
}