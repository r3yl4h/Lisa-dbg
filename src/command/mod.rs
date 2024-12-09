pub mod attach;
pub mod breakpoint;
pub mod def;
pub mod file;
pub mod hook;
pub mod proc_addr;
pub mod remover;
pub mod reset;
pub mod set;
pub mod skip;
pub mod stret;
pub mod sym;
pub mod info;
pub mod watchpoint;

pub mod arg {
    use crate::{usage, ALL_ELM};
    use crate::ut::fmt::{print_lg, LevelPrint};

    pub fn set_argument(linev: &[&str]) {
        if linev.len() < 2 {
            print_lg(LevelPrint::ErrorO, usage::USAGE_SET_ARG.to_string());
            return;
        }
        let arg = linev[1..].join(" ");
        unsafe { ALL_ELM.arg = Some(arg.clone()); }
        print_lg(LevelPrint::DebugO, format!("the arguments have been recorded\narg expression : {arg}"));
    }
}

pub mod with_va {
    use crate::ut::cast::str_to;
    use crate::ut::fmt::{print_lg, LevelPrint};

    pub fn handle_calcule_va(linev: &[&str]) {
        if linev.len() != 2 {
            print_lg(LevelPrint::ErrorO, "USAGE: cva <rva>".to_string());
            return;
        }
        match str_to::<u64>(linev[1]) {
            Ok(value) => {
                print_lg(LevelPrint::DebugO, format!("VA is : {:#x}", unsafe { crate::dbg::BASE_ADDR } + value));
            },
            Err(e) => {
                print_lg(LevelPrint::ErrorO, format!("when transforming rva str '{e}' into u64 : {e}"));
            }
        }
    }

    pub fn handle_calcule_rva(linev: &[&str]) {
        if linev.len() != 2 {
            return;
        }
        match str_to::<u64>(linev[1]) {
            Ok(addr_va) => unsafe {
                if addr_va < crate::dbg::BASE_ADDR {
                    print_lg(LevelPrint::ErrorO, format!(
                        "the specified address cannot be larger than the base address - {:#x} - {:#x}",
                        addr_va, *&raw const crate::dbg::BASE_ADDR
                    ));
                    return;
                }
                print_lg(LevelPrint::DebugO, format!("RVA is : {:#x}", addr_va - crate::dbg::BASE_ADDR));
            },
            Err(e) => {
                print_lg(LevelPrint::ErrorO, format!("{e}"));
            }
        }
    }
}

pub mod clear_cmd {
    use std::process::Command;

    pub fn clear_cmd() {
        Command::new("cmd").args(&["/C", "cls"]).status().unwrap();
    }
}

pub mod little_secret {
    use crate::ut::cast::str_to;
    use crate::ut::fmt::print_lg;
    use crate::ut::fmt::LevelPrint;

    pub fn sub_op(linev: &[&str]) {
        if linev.len() != 3 {
            print_lg(LevelPrint::DebugO, "USAGE: sub <1> <2>      # this is a little secret cmd lol".to_string());
            return;
        }
        match (str_to::<isize>(linev[1]), str_to::<isize>(linev[2])) {
            (Ok(o1), Ok(o2)) => {
                print_lg(LevelPrint::DebugO, format!("result: {:#x}", o1 - o2));
            },
            (Err(o1), Ok(_)) => {
                print_lg(LevelPrint::ErrorO, format!("the first argument is invalid : {o1}"));
            },
            (Ok(_), Err(o2)) => {
                print_lg(LevelPrint::ErrorO, format!("the 2nd argument is invalid : {o2}"));
            },
            (Err(e), Err(e2)) => {
                print_lg(LevelPrint::ErrorO, format!("all argument is invalid, 1 : {e} - 2 : {e2}"));
            }
        }
        print_lg(LevelPrint::DebugO, "".to_string());
    }

    pub fn add_op(linev: &[&str]) {
        if linev.len() != 3 {
            print_lg(LevelPrint::DebugO, "USAGE: add <1> <2>      # this is a little secret cmd lol".to_string());
            return;
        }
        match (str_to::<usize>(linev[1]), str_to::<usize>(linev[2])) {
            (Ok(o1), Ok(o2)) => {
                print_lg(LevelPrint::DebugO, format!("result: {:#x}", o1 + o2));
            },
            (Err(o1), Ok(_)) => {
                print_lg(LevelPrint::ErrorO, format!("the first argument is invalid : {o1}"));
            },
            (Ok(_), Err(o2)) => {
                print_lg(LevelPrint::ErrorO, format!("the 2nd argument is invalid : {o2}"));
            },
            (Err(e), Err(e2)) => {
                print_lg(LevelPrint::ErrorO, format!("all argument is invalid, 1 : {e} - 2 : {e2}"));
            }
        }
        print_lg(LevelPrint::DebugO, "".to_string());
    }
}

