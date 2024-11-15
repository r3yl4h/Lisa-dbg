use std::fmt;
use crate::cli::ALL_ELM;
use crate::dbg::{stop_dbg, DbgState};
use crate::ut::cast::Char;

const ERR_COLOR: &str = "\x1b[31m";
const DBG_COLOR: &str = "\x1b[92m";

pub const GREEN_COL: &str = "\x1b[92m";

pub const RESET_COLOR: &str = "\x1b[0m";
pub const VALUE_COLOR: &str = "\x1b[96m";
pub const VALID_COLOR: &str = "\x1b[32m";
pub const WAR_COLOR: &str = "\x1B[93m";
pub const BYTES_COLOR: &str = "\x1b[93m";
pub const ADDR_COLOR: &str = "\x1b[95m";
pub const SYM_COLOR: &str = "\x1b[94m";
pub const CYAN_COLOR: &str = "\x1b[36m";
pub const MAGENTA: &str = "\x1b[35m";
pub const BLUE_COLOR: &str = "\x1b[34m";

pub const INSTR_COLOR: &str = "\x1b[92m";




pub trait Print {
    fn print_value(&self) -> String;
}

impl Print for u8 {
    fn print_value(&self) -> String {
        format!("{:#x}", self)
    }
}

impl Print for u16 {
    fn print_value(&self) -> String {
        format!("{:#x}", self)
    }
}

impl Print for u32 {
    fn print_value(&self) -> String {
        format!("{:#x}", self)
    }
}

impl Print for u64 {
    fn print_value(&self) -> String {
        format!("{:#x}", self)
    }
}

impl Print for usize {
    fn print_value(&self) -> String {
        format!("{:#x}", self)
    }
}

impl Print for f32 {
    fn print_value(&self) -> String {
        format!("{}", self)
    }
}

impl Print for f64 {
    fn print_value(&self) -> String {
        format!("{}", self)
    }
}

fn format_value<T: fmt::Debug + Default + Copy + PartialOrd + fmt::Display + fmt::LowerHex, >(value: T) -> String {
    if value < T::default() {
        format!("{}", value)
    } else {
        format!("{:#x}", value)
    }
}

impl Print for i8 {
    fn print_value(&self) -> String {
        format_value(*self)
    }
}

impl Print for i16 {
    fn print_value(&self) -> String {
        format_value(*self)
    }
}

impl Print for i32 {
    fn print_value(&self) -> String {
        format_value(*self)
    }
}

impl Print for i64 {
    fn print_value(&self) -> String {
        format_value(*self)
    }
}

impl Print for bool {
    fn print_value(&self) -> String {
        if *self {
            "true".to_string()
        } else {
            "false".to_string()
        }
    }
}

impl Print for char {
    fn print_value(&self) -> String {
        format!("{}", self)
    }
}



pub fn fmt(selfs: &Char, f: &mut fmt::Formatter) -> fmt::Result {
    if selfs.0.is_ascii_graphic() || selfs.0.is_ascii_whitespace() {
        write!(f, "{}", selfs.0 as char)
    } else {
        write!(f, "\\x{:02x}", selfs.0)
    }
}


pub enum LevelPrint {
    Critical1(u32, *mut DbgState),
    Error,
    Debug,
    ErrorO,
    WarningO,
    DebugO
}


pub fn print_lg<F: fmt::Display>(lvl: LevelPrint, fmt: F) {
    match lvl {
        LevelPrint::Critical1(proc, dbg_state) => unsafe {
            eprintln!("[{ERR_COLOR}Critical{RESET_COLOR}] -> {fmt}");
            *dbg_state = DbgState::Stopped;
            stop_dbg(proc)
        }
        LevelPrint::Error => eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> {fmt}"),
        LevelPrint::Debug => unsafe {
            if ALL_ELM.print_ot < 1 {
                println!("[{DBG_COLOR}Debug{RESET_COLOR}] -> {fmt}");
            }
        }
        LevelPrint::ErrorO => eprintln!("{ERR_COLOR}{fmt}{RESET_COLOR}"),
        LevelPrint::WarningO => println!("{WAR_COLOR}{fmt}{RESET_COLOR}"),
        LevelPrint::DebugO => println!("{VALID_COLOR}{fmt}{RESET_COLOR}"),
    }
}