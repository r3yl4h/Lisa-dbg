use crate::ALL_ELM;
use std::ffi::{c_char, CStr};
use std::{mem, ptr};
use crate::dllib::Dll;
use crate::symbol::{SymbolFile, SymbolType, IMAGE_BASE, SYMBOLS_V};
use crate::ut::fmt::{print_lg, LevelPrint};




#[repr(C)]
#[derive(Debug)]
pub struct SymbolsPdb {
    pub size: u32,
    pub value: u64,
    pub address: u64,
    pub tag: u32,
    pub name: *const c_char,
    pub filename: *const c_char,
    pub line: u32,
}

pub unsafe fn target_symbol() {
    match Dll::new("symbol_pe.dll") {
        Ok(dll) => {
            let get_symbol: unsafe extern "C" fn (&mut usize, *const u8) -> *mut SymbolsPdb = {
                match dll.get_func("getSymbols") {
                    Ok(func) => mem::transmute(func),
                    Err(e) => {
                        print_lg(LevelPrint::ErrorO, e);
                        return;
                    } 
                }
            };

            let mut len = 0;
            let file = (*ptr::addr_of_mut!(ALL_ELM)).file.clone().unwrap();
            let res = get_symbol(&mut len, format!("{file}\0").as_ptr());
            if len != 0 {
                SYMBOLS_V.symbol_type = SymbolType::PDB;
            } else {
                return;
            }
            let symv = std::slice::from_raw_parts(res, len);
            let get_tag_str: unsafe extern "C" fn(u32) -> *const u8 = {
                match dll.get_func("GetTagString") {
                    Ok(func) => mem::transmute(func),
                    Err(e) => {
                        print_lg(LevelPrint::ErrorO, e);
                        return
                    },
                }
            };
            push_pdb(symv, get_tag_str);
            let free_symbols: unsafe extern "C" fn(*mut SymbolsPdb, usize) = {
                match dll.get_func("freeSymbols") {
                    Ok(func) => mem::transmute(func),
                    Err(e) => {
                        print_lg(LevelPrint::ErrorO, e);
                        return
                    }
                }
            };
            free_symbols(res, len);
        }
        Err(e) => print_lg(LevelPrint::ErrorO, e)
    }
}



fn push_pdb(sym: &[SymbolsPdb], get_tag_str: unsafe extern "C" fn(u32) -> *const u8) {
    unsafe {
        for sym in sym {
            let mut sym_e = SymbolFile::default();
            if IMAGE_BASE < sym.address {
                sym_e.offset = (sym.address - IMAGE_BASE) as i64;
            } else {
                sym_e.offset = sym.address as i64;
            }
            sym_e.size = sym.size as usize;
            sym_e.types_e = CStr::from_ptr(get_tag_str(sym.tag) as *const i8).to_string_lossy().to_string();
            sym_e.name = CStr::from_ptr(sym.name).to_string_lossy().to_string();
            sym_e.filename = CStr::from_ptr(sym.filename).to_string_lossy().to_string();
            sym_e.line = sym.line as usize;
            (*ptr::addr_of_mut!(SYMBOLS_V.symbol_file)).push(sym_e);
        }
    }
}


pub fn from_pdb_file(pdb_file: &str) -> Result<(), anyhow::Error>{
    unsafe {
        let symbol_pe = Dll::new("symbol_pe.dll")?;
        let get_pdb_sym: unsafe extern "C" fn(*const u8, &mut usize, u64) -> *mut SymbolsPdb = mem::transmute(symbol_pe.get_func("GetPdbSym")?);

        let mut len = 0;
        let res = get_pdb_sym(pdb_file.as_ptr(), &mut len, IMAGE_BASE);
        if len != 0 {
            SYMBOLS_V.symbol_type = SymbolType::PDB;
        } else {
            return Ok(());
        }
        let symv = std::slice::from_raw_parts(res, len);
        let get_tag_str: unsafe extern "C" fn(u32) -> *const u8 = mem::transmute(symbol_pe.get_func("GetTagString")?);
        push_pdb(symv, get_tag_str);
        let free_symbols: unsafe extern "C" fn(*mut SymbolsPdb, usize) = mem::transmute(symbol_pe.get_func("freeSymbols")?);
        free_symbols(res, len);
    }
    Ok(())
}



/*pub fn get_reg_with_reg_field(reg_field: u32) -> String {
    let res = match reg_field {
        17 => "eax",
        18 => "ecx",
        19 => "edx",
        20 => "ebx",
        21 => "esp",
        22 => "ebp",
        23 => "esi",
        24 => "edi",
        328 => "rax",
        329 => "rbx",
        330 => "rcx",
        331 => "rdx",
        332 => "rsi",
        333 => "rdi",
        334 => "rbp",
        335 => "rsp",
        336 => "r8",
        337 => "r9",
        338 => "r10",
        339 => "r11",
        340 => "r12",
        341 => "r13",
        342 => "r14",
        343 => "r15",
        _ => "",
    };
    res.to_string()
}*/


