use crate::{pefile, symbol, ALL_ELM};
use std::fs::File;
use std::io::Read;
use std::ptr;
use crate::symbol::{Symbols, SYMBOLS_V};
use crate::ut::fmt::{print_lg, LevelPrint};

pub fn handle_change_file(linev: &[&str], line: &str) {
    if linev.len() > 1 {
        let file_str = line[4..].replace("\"", "");
        let file_str = file_str.trim();
        match File::open(file_str) {
            Ok(mut file) => {
                unsafe {
                    let mut mz_head = [0u8; 2];
                    file.read_exact(&mut mz_head).unwrap();
                    if mz_head == *b"MZ" {
                        ALL_ELM.file = Some(file_str.to_string());
                        **(ptr::addr_of_mut!(SYMBOLS_V)) = Symbols::default();
                        if let Err(e) = pefile::parse_header() {
                            print_lg(LevelPrint::ErrorO, e);
                            return;
                        }
                        print_lg(LevelPrint::DebugO, format!("Now the file context is '{file_str}'"));
                        symbol::load_symbol(linev, line);
                    } else {
                        print_lg(LevelPrint::ErrorO, "please specify a valid pe file");
                    }
                }
            }
            Err(e) => print_lg(LevelPrint::ErrorO, format!("failed to open file: {e}")),
        }
    }else {
        print_lg(LevelPrint::DebugO, "USAGE: file <path>");
    }
}
