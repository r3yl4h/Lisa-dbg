use crate::usage::USAGE_DEF;
use crate::ut::fmt::{print_lg, LevelPrint};

pub mod func;
pub mod structs;
pub mod variable;
pub mod types;

pub fn handle_def(linev: &[&str], line: &str) {
    if linev.len() < 2 {
        println!("{USAGE_DEF}");
        return;
    }
    let type_elm = linev[1];
    match type_elm {
        "func" | "function" => func::crt_func(&linev[1..], line[type_elm.len()..].trim()),
        "struct" => structs::def_struct(&linev[1..]),
        "var" | "variable" => variable::variable::handle_var(&linev[2..]),
        _ => print_lg(LevelPrint::ErrorO, format!("unknow element '{type_elm}'")),
    }
}
