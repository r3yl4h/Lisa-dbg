extern crate core;

mod cli;
mod command;
mod dbg;
mod pefile;
mod process;
mod ste;
mod symbol;
mod usage;
mod ut;

use crate::cli::ALL_ELM;
use crate::command::def;
use std::io;
use std::io::Write;
use structopt::StructOpt;
use winapi::um::winnt::HANDLE;
use command::def::variable;
use crate::ut::fmt::{print_lg, LevelPrint};

fn main() {
    let option = cli::Dbgoption::from_args();
    unsafe {*ALL_ELM = option.to_all_elm();}
    if let Some(file) = &option.file {
        let intp = format!("file {file}");
        command::file::handle_change_file(&intp.split_whitespace().collect::<Vec<&str>>(), &intp);
    }
    option.exec_cmd();
    ctx_before_run();
}


fn ctx_before_run() {
    loop {
        let mut input = String::new();
        print!(">> ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim_start().trim_end();
        let linev: Vec<&str> = input.split_whitespace().collect();
        handle_cmd(&linev, input);
    }
}



fn handle_cmd(linev: &[&str], input: &str) {
    let cmd = linev.first();
    match cmd {
        Some(&"breakpoint") | Some(&"b") => command::breakpoint::handle_breakpts(&linev),
        Some(&"file") => command::file::handle_change_file(&linev, input),
        Some(&"run") => dbg::run(),
        Some(&"reset") => command::reset::handle_reset(&linev),
        Some(&"quit") | Some(&"q") | Some(&"exit") => std::process::exit(0),
        Some(&"s") | Some(&"sym") | Some(&"symbol") => symbol::load_symbol(),
        Some(&"break-ret") | Some(&"b-ret") => command::stret::st_return(&linev),
        Some(&"skip") => command::skip::skip(&linev),
        Some(&"hook") | Some(&"ho") => command::hook::hook(&linev),
        Some(&"def") => def::handle_def(&linev),
        Some(&"arg") | Some(&"args") | Some(&"argv") => command::arg::set_argument(&linev),
        Some(&"help") | Some(&"h") => usage::help(&linev),
        Some(&"help-c") => dbg::dbg_cmd::usages::help(&linev),
        Some(&"info") => unsafe { command::info::handle_info(&linev, std::mem::zeroed(), 0 as HANDLE) },
        Some(&"w") | Some(&"watch") | Some(&"watchpoint") => command::watchpoint::watchpoint(&linev),
        Some(&"clear") => command::clear_cmd::clear_cmd(),
        Some(&"remove") => command::remover::remove_element(&linev),
        Some(&"sym-info") => unsafe { command::sym::handle_sym_info(&linev, std::mem::zeroed()) },
        Some(&"attach") => command::attach::handle_attach(&linev),
        Some(&"bva") | Some(&"b-va") | Some(&"break-va") => command::breakpoint::handle_break_va(&linev),
        Some(&"proc-addr") => command::proc_addr::handle_get_proc_addr(linev),
        Some(&"b-ret-va") | Some(&"b-retva") => command::stret::handle_b_ret_va(&linev),
        Some(&"add") => command::little_secret::add_op(&linev),
        Some(&"sub") => command::little_secret::sub_op(&linev),
        Some(&"printf") => variable::printf::printf_var(&linev, input, 0 as HANDLE),
        None => print_lg(LevelPrint::ErrorO, "please enter a command"),
        _ => print_lg(LevelPrint::ErrorO, format!("command '{}' is unknow", cmd.unwrap())),
    }
}
