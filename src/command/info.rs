use crate::ut::fmt::VALID_COLOR;
use crate::dbg::{memory, RealAddr, BASE_ADDR};
use crate::pefile::function::FUNC_INFO;
use crate::pefile::section::SECTION_VS;
use crate::symbol::SYMBOLS_V;
use crate::usage;
use std::ffi::CStr;
use std::ptr::addr_of;
use std::{io, mem, ptr};
use winapi::um::processthreadsapi::GetProcessId;
use winapi::um::tlhelp32::{
    CreateToolhelp32Snapshot, Thread32First, Thread32Next,
    TH32CS_SNAPTHREAD, THREADENTRY32,
};
use winapi::um::winnt::{CONTEXT, HANDLE, RUNTIME_FUNCTION};
use crate::cli::ALL_ELM;
use crate::command::breakpoint::Brkpts;
use crate::process::get_module;
use crate::ut::fmt::*;

pub fn handle_info(linev: &[&str], ctx: *const CONTEXT, proc_handle: HANDLE) {
    if linev.len() < 2 {
        println!("{}", usage::USAGE_INFO);
        return;
    }
    let elm = linev[1];
    match elm {
        "breakpoint" | "brpt" | "b" => print_elements(unsafe { &*addr_of!(ALL_ELM.break_rva) }),
        "skip" => print_elements(unsafe { &*addr_of!(ALL_ELM.skip_addr) }),
        "b-ret" => print_elements(unsafe { &*addr_of!(ALL_ELM.break_ret) }),
        "symbol" | "sym" | "s" => print_sym(&linev[1..], ctx),
        "hook-func" | "hook" | "h" => print_hook_func(),
        "watchpoint" | "watch" | "w" => print_watchpt(ctx),
        "function" | "func" | "f" => print_function(),
        "section" | "sec" => print_section(),
        "break-va" | "bva" | "b-va" => print_elements(unsafe { &*addr_of!(ALL_ELM.break_va) }),
        "break-ret-va" | "b-ret-va" => print_elements(unsafe { &*addr_of!(ALL_ELM.break_ret_va) }),
        "proc" => view_all_module(proc_handle),
        "hmodule" | "module" | "m" => handle_module(&linev[1..], proc_handle),
        "def" => info_def(&linev[1..]),
        "thread" | "th" => view_thread(proc_handle),
        "register" | "reg" => print_lg(LevelPrint::ErrorO, "to see the value of a register, type \"reg <register>\" or \"value <register>\""),
        "file" => {
            print!("{VALID_COLOR}");
            unsafe {
                if let Some(file) = &(*addr_of!(ALL_ELM)).file {
                    print!("{file}");
                } else {
                    print!("not specified");
                }
            }
            println!("{RESET_COLOR}");
        },
        _ => print_lg(LevelPrint::ErrorO, format!("unknow option : '{elm}'")),
    }
}



pub fn handle_module(linev: &[&str], h_proc: HANDLE) {
    if h_proc.is_null() {
        print_start();
        return;
    }

    if linev.len() < 2 || linev[1] == "map" {
        view_all_module(h_proc)
    }
    else {
        match get_module(h_proc) {
            Ok(module) => unsafe {
                println!(
                    "{}{:<10}{} {}{:<25}{} {}{:<25}{} {}{:<15}{} {}{:<5}{}",
                    "\x1b[32m", "PID", "\x1b[0m",
                    "\x1b[33m", "Start Addr", "\x1b[0m",
                    "\x1b[33m", "End Addr", "\x1b[0m",
                    "\x1b[36m", "Size", "\x1b[0m",
                    "\x1b[35m", "Module", "\x1b[0m"
                );
                
                let print_f = linev.len() > 2 && linev[2].contains("func");
                
                for entry32 in module {
                    if linev[1] == CStr::from_ptr(entry32.szModule.as_ptr()).to_string_lossy() {
                        let end_addr = entry32.modBaseAddr as u64 + entry32.modBaseSize as u64;
                        let module_name = CStr::from_ptr(entry32.szModule.as_ptr()).to_string_lossy();
                        let base = entry32.modBaseAddr as u64;
                        println!(
                            "{}{:<10}{} {}{:#018x}{}       {}{:#018x}{}       {}{:<#10x}{} {}{:<30}{}",
                            BLUE_COLOR, entry32.th32ProcessID, RESET_COLOR,
                            "\x1b[33m", base, RESET_COLOR,
                            "\x1b[33m", end_addr, RESET_COLOR,
                            "\x1b[36m", entry32.modBaseSize, RESET_COLOR,
                            "\x1b[35m", module_name, RESET_COLOR
                        );
                        if print_f {
                            let s = (*addr_of!(SYMBOLS_V)).symbol_file.iter().filter(|s|s.is_in_this_dll(base)).collect::<Vec<_>>();
                            println!();
                            println!(
                                "{}{:<25}{} {}{:<25}{}",
                                "\x1b[36m", "Start Addr", "\x1b[0m",
                                "\x1b[35m", "Function", "\x1b[0m"
                            );
                            for s in s {
                                println!(
                                    "{}{:#018x}{}   ->  {}{:<30}{}",
                                    "\x1b[36m", s.offset as u64 + s.src_file.dll_base(), RESET_COLOR,
                                    "\x1b[35m", s.name, RESET_COLOR
                                );
                            }
                        }
                        return
                    }
                }
                print_lg(LevelPrint::ErrorO, format!("unknow module named '{}'", linev[1]))
            }
            Err(e) => print_lg(LevelPrint::ErrorO, format!("Error getting module: {}", e)),
        }
    }
}








pub fn view_thread(proc_handle: HANDLE) {
    if proc_handle.is_null() {
        print_start();
        return;
    }
    let pid = unsafe { GetProcessId(proc_handle) };
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, pid) };
    let mut cout = 0;
    if snapshot.is_null() {
        print_lg(LevelPrint::ErrorO, format!("failed to create tool snapshot : {}", io::Error::last_os_error()));
        return;
    }
    unsafe {
        let mut th32: THREADENTRY32 = mem::zeroed();
        th32.dwSize = size_of::<THREADENTRY32>() as u32;
        if Thread32First(snapshot, &mut th32) == 0 {
            print_lg(LevelPrint::ErrorO, format!("Failed to get first thread32 : {}", io::Error::last_os_error()));
            return;
        }

        loop {
            if th32.th32OwnerProcessID == pid {
                print_thread(th32, &mut cout);
            }
            if Thread32Next(snapshot, &mut th32) == 0 {
                if io::Error::last_os_error().raw_os_error() != Some(18) {
                    print_lg(LevelPrint::ErrorO, format!("Failed to get next thread32 : {}", io::Error::last_os_error()));
                }
                return;
            }
        }
    }
}

fn print_thread(th32: THREADENTRY32, cout: &mut i32) {
    println!("{}Thread{cout}: ", GREEN_COL);
    println!("    {}Thread id : {}", MAGENTA, th32.th32ThreadID);
    println!("    {}Owner pid : {}", VALUE_COLOR, th32.th32OwnerProcessID);
    println!("    {}Base prlvl: {}{RESET_COLOR}", BYTES_COLOR, th32.tpBasePri);
    println!();
    *cout += 1;
}

fn info_def(arg: &[&str]) {
    if arg.len() < 2 {
        print_cr_func(&[""]);
        print_struct_def(&[""]);
        return;
    }
    let type_elm = arg[1];
    match type_elm {
        "function" | "func" => print_cr_func(&arg[1..]),
        "struct" | "structure" => print_struct_def(&arg[1..]),
        "variable" | "var" => {},
        _ => print_lg(LevelPrint::ErrorO, format!("unknow option : '{type_elm}'"))
    }
}

fn print_struct_def(arg: &[&str]) {
    if arg.len() < 2 {
        unsafe {
            println!("number of structure : {}", &(*addr_of!(ALL_ELM)).struct_def.len());
            for structs in &(*addr_of!(ALL_ELM)).struct_def {
                println!("  struct {};", structs.get_name_of_struct());
            }
        }
    }else {
        if let Some(structs) = unsafe {&(*addr_of!(ALL_ELM)).struct_def.iter().find(|s|s.get_name_of_struct() == arg[1])}{
            println!("\nstruct {} {{", structs.get_name_of_struct());
            for field in structs.get_field_of_struct() {
                println!("    {} {};", field.type_p, field.name_field);
            }
            println!("}}");
        }
    }
}



fn print_cr_func(arg: &[&str]) {
    if arg.len() < 2 {
        unsafe {
            print_lg(LevelPrint::DebugO, format!("Number of function : {}", (*addr_of!(ALL_ELM)).crt_func.len()));
            for func in &(*addr_of!(ALL_ELM)).crt_func {
                println!("\n{:#x}: {}", func.addr, func.name);
            }
        }
    } else {
        if let Some(cr_func) = unsafe { (*addr_of!(ALL_ELM)).crt_func.iter().find(|s| s.name == arg[1]) } {
            println!("{} {}: ", ADDR_COLOR, cr_func.name);
            for line in &cr_func.code_str {
                println!("     {line}");
            }
        } else {
            print_lg(LevelPrint::ErrorO, format!("Unknow def function '{}'", arg[1]));
        }
        println!("{RESET_COLOR}");
    }
}

fn print_start() {
    print_lg(LevelPrint::WarningO, "you must have started the process to be able to use this option");
}






fn view_all_module(h_proc: HANDLE) {
    match get_module(h_proc) {
        Ok(modules) => unsafe {
            println!(
                "{}{:<10}{} {}{:<25}{} {}{:<25}{} {}{:<15}{} {}{:<5}{}",
                "\x1b[32m", "PID", "\x1b[0m",
                "\x1b[33m", "Start Addr", "\x1b[0m",
                "\x1b[33m", "End Addr", "\x1b[0m",
                "\x1b[36m", "Size", "\x1b[0m",
                "\x1b[35m", "Module", "\x1b[0m"
            );

            for entry32 in modules {
                let end_addr = entry32.modBaseAddr as u64 + entry32.modBaseSize as u64;
                let module_name = CStr::from_ptr(entry32.szModule.as_ptr()).to_string_lossy();

                println!(
                    "{}{:<10}{} {}{:#018x}{}       {}{:#018x}{}       {}{:<#10x}{} {}{:<30}{}",
                    BLUE_COLOR, entry32.th32ProcessID, RESET_COLOR,
                    "\x1b[33m", entry32.modBaseAddr as u64, RESET_COLOR,
                    "\x1b[33m", end_addr, RESET_COLOR,
                    "\x1b[36m", entry32.modBaseSize, RESET_COLOR,
                    "\x1b[35m", module_name, RESET_COLOR
                );
            }
        }
        Err(e) => print_lg(LevelPrint::ErrorO, e),
    }
}



fn print_section() {
    unsafe {
        for (i, section) in (*addr_of!(SECTION_VS)).iter().enumerate() {
            println!("\n{VALID_COLOR}#{i}: \
            \n     {}Name         : {}\
            \n     {}Address      : {:#x}\
            \n     {}Size of code : {:#x}{RESET_COLOR}",
                GREEN_COL, section.name,
                ADDR_COLOR, section.addr as u64 + BASE_ADDR,
                VALUE_COLOR, section.content.len()
            )
        }
    }
}

fn print_function() {
    unsafe {
        for (i, func) in (&*addr_of!(FUNC_INFO)).iter().enumerate() {
            println!(
                "\n{VALID_COLOR}func_#{i}:\
        \n     {}Address     : {:#x} {}\
        \n     {}end-address : {:#x}\
        \n     {}size        : {:#x}{RESET_COLOR}",
                ADDR_COLOR, func.BeginAddress as u64 + BASE_ADDR, get_sym_name(func),
                VALUE_COLOR, func.EndAddress as u64 + BASE_ADDR,
                MAGENTA, func.EndAddress - func.BeginAddress,
            )
        }
    }
}

fn get_sym_name(func: &RUNTIME_FUNCTION) -> String {
    if let Some(sym) = unsafe {
        (*addr_of!(SYMBOLS_V)).symbol_file.iter().find(|s| s.offset + BASE_ADDR as i64 == func.BeginAddress as i64 + BASE_ADDR as i64)
    } {
        format!("<{}>", sym.name)
    } else {
        "".to_string()
    }
}

fn print_elements<T: IntoIterator<Item = &'static Brkpts>>(elements: T) {
    for (i, e) in elements.into_iter().enumerate() {
        println!("{i} : {:#x}", e.addr);
    }
}



fn print_watchpt(ctx: *const CONTEXT) {
    unsafe {
        for (i, watchpts) in (*addr_of!(ALL_ELM)).watchpts.iter().enumerate() {
            println!(
                "{GREEN_COL}{i}: \
        \n     {}memory zone    : {}\
        \n     {}check access   : {:?}\
        \n     {}offset         : {}\
        \n     {}size           : {:#x}{RESET_COLOR}",
                CYAN_COLOR, watchpts.flag_type_mem,
                BYTES_COLOR, watchpts.check_type,
                ADDR_COLOR, watchpts.format_offset(ctx),
                VALID_COLOR, watchpts.memory_size
            );
        }
    }
}

pub fn print_hook_func() {
    unsafe {
        for (i, hook) in (*addr_of!(ALL_ELM)).hook.iter().enumerate() {
            println!(
                "{VALUE_COLOR}{i}{RESET_COLOR}:\
            \n     {WAR_COLOR}Target   : {GREEN_COL}{:#x}\
            \n     {WAR_COLOR}Replace  : {MAGENTA}{:#x}{RESET_COLOR}\n",
                hook.target, hook.replacen
            );
        }
    }
}

pub fn print_sym(linev: &[&str], ctx: *const CONTEXT) {
    println!("{VALID_COLOR}Symbol type: {VALUE_COLOR}{}{RESET_COLOR}", unsafe { (*ptr::addr_of!(SYMBOLS_V)).symbol_type });
    if linev.len() > 1 {
        if let Some(sym) = unsafe { (*addr_of!(SYMBOLS_V)).symbol_file.iter().find(|s|s.name == linev[1])} {
            println!(
                "{CYAN_COLOR}1{RESET_COLOR}:\
            \n     {}Name     : {}\
            \n     {}\
            \n     {}Type     : {}\
            \n     {}Size     : {:#x}\
            \n     {}file     : {}:{}\
            {RESET_COLOR}\n",
                GREEN_COL, sym.name,
                if unsafe { BASE_ADDR == 0 } {
                    format!("{}offset   : {:#x}", ADDR_COLOR, sym.offset)
                } else {
                    format!("{}address  : {:#x}", ADDR_COLOR, sym.real_addr(ctx))
                },
                BLUE_COLOR, sym.types_e,
                MAGENTA, sym.size,
                WAR_COLOR, sym.filename, sym.line,
            );
        } else {
            print_lg(LevelPrint::ErrorO, "unknown symbol");
        }
    }else {
        for (i, sym) in unsafe { (*addr_of!(SYMBOLS_V)).symbol_file.iter().enumerate() } {
            println!(
                "{CYAN_COLOR}{i}{RESET_COLOR}:\
            \n     {}Name     : {}\
            \n     {}\
            \n     {}Type     : {}\
            \n     {}Size     : {:#x}\
            \n     {}file     : {}:{}\
            {RESET_COLOR}\n",
                GREEN_COL, sym.name,
                if unsafe { BASE_ADDR == 0 } {
                    format!("{}offset   : {:#x}", ADDR_COLOR, sym.offset)
                } else {
                    format!("{}address  : {:#x}", ADDR_COLOR, sym.real_addr(ctx))
                },
                BLUE_COLOR, sym.types_e,
                MAGENTA, sym.size,
                WAR_COLOR, sym.filename, sym.line,
            );
        }
    }
}



pub fn print_frame(count: usize, ctx: *const CONTEXT) {
    unsafe {
        for i in 0..count {
            if let Some(frame) = (*addr_of!(memory::stack::ST_FRAME)).get(i) {
                let get_function_and_symbol = |offset| {
                    if let Some(sym) = (*&raw const SYMBOLS_V).symbol_file.iter().find(|s|s.is_in_sym(offset, ctx)) {
                        let offset = offset - sym.real_addr(ctx);
                        return format!("<{}+{}>", sym.name, offset);
                    }
                            else if let Some(f) = (*&raw const FUNC_INFO).iter().find(|f|{
                        f.BeginAddress as u64 + BASE_ADDR <= offset && f.EndAddress as u64 + BASE_ADDR >= offset
                    }) {
                        let func_addr = f.BeginAddress as u64 + BASE_ADDR;
                        let offset = offset - func_addr;
                        return format!("<func_{:#x}+{}>", func_addr - BASE_ADDR, offset);
                    }
                    return String::from("")
                };
                println!("\n{}#{}:", BLUE_COLOR, i);
                println!("{}   rip               = {}{:#18x} {}", ADDR_COLOR, VALUE_COLOR, frame.AddrPC.Offset, get_function_and_symbol(frame.AddrPC.Offset));
                println!("{}   Return Address    = {}{:#18x} {}", ADDR_COLOR, BYTES_COLOR, frame.AddrReturn.Offset, get_function_and_symbol(frame.AddrReturn.Offset));
                println!("{}   Frame Ptr         = {}{:#18x}", ADDR_COLOR, SYM_COLOR, frame.AddrFrame.Offset);
                println!("{}   Stack Ptr         = {}{:#18x}", ADDR_COLOR, GREEN_COL, frame.AddrStack.Offset);
            } else {
                if count != usize::MAX {
                    print_lg(LevelPrint::WarningO, format!("the count is greater than the total number of frames, frame: {} count: {}", (*&raw const memory::stack::ST_FRAME).len(), count));
                }
                return;
            }
        }
    }
}
