use crate::{pefile, symbol, ALL_ELM};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::ptr;
use crate::symbol::SYMBOLS_V;
use crate::ut::fmt::{print_lg, LevelPrint};

pub fn handle_change_file(linev: &[&str], line: &str) {
    if linev.len() > 1 {
        let file_str = line[4..].replace("\"", "");
        let file_str = file_str.trim_start().trim_end();
        if Path::new(file_str).exists() {
            unsafe {
                let mut file = File::open(file_str).unwrap();
                let mut mz_head = [0u8; 2];
                file.read_exact(&mut mz_head).unwrap();
                if mz_head == *b"MZ" {
                    ALL_ELM.file = Some(file_str.to_string());
                    (*ptr::addr_of_mut!(SYMBOLS_V)).symbol_file.clear();
                    if let Err(e) = pefile::parse_header() {
                        print_lg(LevelPrint::ErrorO, e);
                        return;
                    }
                    print_lg(LevelPrint::DebugO, format!("Now the file context is '{file_str}'"));
                    symbol::load_symbol();
                } else {
                    print_lg(LevelPrint::ErrorO, "please specify a valid pe file");
                }
            }
        } else {
            print_lg(LevelPrint::ErrorO, format!("the path {file_str} doesn't exist"));
        }
    }else {
        print_lg(LevelPrint::DebugO, "USAGE: file <path>");
    }
}
