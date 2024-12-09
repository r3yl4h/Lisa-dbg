use crate::command::def;
use crate::command::def::func::CrtFunc;
use crate::command::hook::Hook;
use crate::command::watchpoint::Watchpts;
use crate::{command, handle_cmd};
use once_cell::sync::Lazy;
use structopt::StructOpt;
use crate::command::breakpoint::Brkpts;
use crate::dbg::BASE_ADDR;

#[derive(Debug, Default, Copy, Clone)]
pub struct AfterB {
    pub last_addr_b: u64,
    pub after_b: u64,
    pub last_oc: u8,
}

#[derive(Debug, Default)]
pub struct All {
    pub file: Option<String>,
    pub break_rva: Vec<Brkpts>,
    pub break_va: Vec<Brkpts>,
    pub arg: Option<String>,
    pub watchpts: Vec<Watchpts>,
    pub skip_addr: Vec<Brkpts>,
    pub crt_func: Vec<CrtFunc>,
    pub break_ret: Vec<Brkpts>,
    pub hook: Vec<Hook>,
    pub attach: Option<String>,
    pub struct_def: Vec<def::types::TypeP>,
    pub after_b: Vec<AfterB>,
    pub var_def: Vec<def::variable::Var>,
    pub break_ret_va: Vec<Brkpts>,
    pub print_ot: u8,
    pub pdb_path: Option<String>,
}

impl All {
    pub fn break_contain(&self, addr: u64) -> bool {
        unsafe {
            self.break_rva.iter().any(|brkpt| brkpt.addr == addr || brkpt.addr == addr)
                | self.break_va.iter().any(|brkpt| brkpt.addr == addr || brkpt.addr == addr + BASE_ADDR)
                | self.break_ret.iter().any(|brkpt| brkpt.addr == addr)
                | self.break_ret_va.iter().any(|brkpt| brkpt.addr == addr || brkpt.addr == addr + BASE_ADDR)
        }
    }
    
    pub fn find_b_rva_with_addr(&self, addr: u64) -> Option<&Brkpts> {
        if let Some(b) = self.break_rva.iter().find(|brkpt| brkpt.addr == addr) {
            Some(b)
        } else if let Some(b) = self.break_ret.iter().find(|brkpt| brkpt.addr == addr) {
            Some(b)
        } else {
            None
        }
    }
}


pub static mut ALL_ELM: Lazy<All> = Lazy::new(|| All::default());



#[derive(Debug, StructOpt, Default)]
#[structopt(name = "LisaDbg", version = "1.0")]
pub struct Dbgoption {
    pub file: Option<String>,
    #[structopt(short = "b", long = "breakpoint", help = "to place a breakpoint at an address (RVA)")]
    breakpoint_addr: Vec<u64>,
    #[structopt(long = "b-ret-va", help = "to place a breakpoint at ret addr of the function which contain the va")]
    b_ret_va: Vec<u64>,
    #[structopt(long = "b-ret", help = "to place a breakpoint at ret addr of the function which contain the rva")]
    b_ret: Vec<u64>,
    #[structopt(long = "b-va", help = "to place a breakpoint at an address (VA) you must know in advance the address going and")]
    b_va: Vec<u64>,
    #[structopt(short = "a", long = "arg", help = "set arguments for script to debug")]
    arg: Option<String>,
    #[structopt(long = "exec", help = "to execute a cmd specified before running dbg")]
    exec_cmd: Vec<String>,
    #[structopt(short = "w", long = "watchpoint", help = "Set a watchpoint in the format '[--memory=<zone>] [--access=<rights>] <offset>")]
    watchpts: Vec<Watchpts>,
    #[structopt(long = "attach", help = "attach the dbg of a existing process with here pid or here name")]
    attach: Option<String>,
}

impl Dbgoption {
    pub fn exec_cmd(&self) {
        for cmd in &self.exec_cmd {
            let linev: Vec<&str> = cmd.split_whitespace().collect();
            handle_cmd(&linev, &cmd);
        }
        if let Some(at_str) = &self.attach {
            let line = format!("attach {at_str}");
            command::attach::handle_attach(&line.split_whitespace().collect::<Vec<&str>>())
        }
    }

    pub fn to_all_elm(&self) -> All {
        let mut result = All::default();
        result.file = self.file.clone();
        set_brkpts(&mut result.break_rva, &self.breakpoint_addr);
        result.arg = self.arg.clone();
        result.watchpts = self.watchpts.clone();
        set_brkpts(&mut result.break_va, &self.b_va);
        set_brkpts(&mut result.break_ret, &self.b_ret);
        set_brkpts(&mut result.break_ret_va, &self.b_ret_va);
        set_brkpts(&mut result.break_ret, &self.b_va);
        result
    }
}


pub fn set_brkpts(dest: &mut Vec<Brkpts>, source: &[u64]) {
    for b in source {
        dest.push(Brkpts::from_addr_no_start(*b));
    }
}