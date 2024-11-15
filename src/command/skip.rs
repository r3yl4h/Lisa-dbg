use crate::usage;
use crate::ut::fmt::{print_lg, LevelPrint};

pub fn skip(linev: &[&str]) {
    if linev.len() < 2 {
        eprintln!("{}", usage::USAGE_SKIP);
        return;
    }
    let addr_func = match crate::ste::get_address(linev) {
        Ok(addr) => addr,
        Err(e) => {
            print_lg(LevelPrint::ErrorO, e);
            return;
        }
    };

    match crate::ste::find_func_by_addr(addr_func) {
        Some(_) => {
            print_lg(LevelPrint::DebugO, format!("the function {:#x} will now not be executed", addr_func));
        }
        None => print_lg(LevelPrint::ErrorO, format!("unknow target : '{:#x}'", addr_func)),
    }
}
