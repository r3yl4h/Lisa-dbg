mod dwarf;
pub mod pdb;
use crate::dbg::{memory, RealAddr, BASE_ADDR};
use crate::{pefile, ALL_ELM};
use std::cmp::PartialEq;
use std::{fmt, io, mem, ptr};
use std::fmt::Formatter;
use anyhow::anyhow;
use once_cell::sync::Lazy;
use winapi::shared::minwindef::BOOL;
use winapi::shared::ntdef::HANDLE;
use winapi::um::winnt::{CONTEXT, WOW64_CONTEXT};
use crate::dllib::Dll;
use crate::pefile::{NtHeaders, NT_HEADER};
use crate::ut::fmt::*;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SymbolType {
    DWARF,
    PDB,
    Un,
}

impl fmt::Display for SymbolType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            SymbolType::Un => write!(f, "UNKNOW"),
            SymbolType::DWARF => write!(f, "DWARF"),
            SymbolType::PDB => write!(f, "PDB"),
        }
    }
}

impl Default for SymbolType {
    fn default() -> Self {
        SymbolType::Un
    }
}



#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SymType {
    Global,
    Local,
    // flemme pr les autres en v2v
}

impl Default for SymType {
    fn default() -> Self {
        SymType::Global
    }
}


#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct SymbolFile {
    pub name: String,
    pub offset: i64,
    pub size: usize,
    pub value_str: String,
    pub types_e: String,
    pub filename: String,
    pub line: usize,
    pub symbol_type: SymType,
    pub src_file: SrcFile
}


#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SrcFile {
    Dll(u64),
    Ex,
}


impl SrcFile {
    pub fn is_dll(&self) -> bool {
        match self {
            SrcFile::Dll(_) => true,
            _ => false
        }
    }
    
    pub fn dll_base(&self) -> u64 {
        match self {
            SrcFile::Dll(base) => *base,
            _ => 0,
        }
    }
}

impl Default for SrcFile {
    fn default() -> Self {
        SrcFile::Ex
    }
}


impl SymbolFile {
    pub fn is_in_sym(&self, addr: u64, ctx: *const CONTEXT) -> bool {
        let start_addr = self.real_addr(ctx);
        let end_addr = start_addr + self.size as u64;
        addr >= start_addr && addr <= end_addr
    }
    
    pub fn is_in_this_dll(&self, base: u64) -> bool {
        if self.src_file.is_dll() {
            self.src_file.dll_base() == base
        }else {
            false
        }
    }
    
     pub fn addr_ot(&self, ctx: *const CONTEXT) -> u64 {
         if ctx.is_null() {
             if self.offset > 0 {
                 self.offset as u64
             } else {
                 0
             }
         }else {
             self.real_addr(ctx)
         }
     }
}

impl RealAddr for SymbolFile {
    fn real_addr64(&self, ctx: CONTEXT) -> u64 {
        if self.offset < 0 && unsafe { (*&raw const SYMBOLS_V).symbol_type == SymbolType::DWARF }{
            unsafe {
                if let Some(b_frame) = memory::stack::get_frame_before_func(ctx.Rip) {
                    (b_frame.AddrStack.Offset as i64 + self.offset) as u64
                } else {
                    print_lg(LevelPrint::Error, "failed to get last frame before current frame".to_string());
                    0
                }
            }
        }else {
            match self.symbol_type {
                SymType::Global => {
                    if self.src_file.is_dll(){
                        self.src_file.dll_base() + self.offset as u64
                    } else {
                        unsafe { BASE_ADDR + self.offset as u64 }
                    }
                }
                SymType::Local => self.offset as u64
            }
        }
    }


    fn real_addr32(&self, ctx: WOW64_CONTEXT) -> u32 {
        if self.offset < 0 && unsafe { (*&raw const SYMBOLS_V).symbol_type == SymbolType::DWARF }{
            unsafe {
                if let Some(b_frame) = memory::stack::get_frame_before_func(ctx.Eip as u64) {
                    (b_frame.AddrStack.Offset as i64 + self.offset) as u32
                } else {
                    print_lg(LevelPrint::Error, "failed to get last frame before current frame".to_string());
                    0
                }
            }
        }else {
            match self.symbol_type {
                SymType::Global => {
                    if self.src_file.is_dll(){
                        (self.src_file.dll_base() + self.offset as u64) as u32
                    } else {
                        unsafe { BASE_ADDR as u32 + self.offset as u32 }
                    }
                }
                SymType::Local => self.offset as u32
            }
        }
    }


    fn real_addr(&self, ctx: *const CONTEXT) -> u64 {
        unsafe {
            match NT_HEADER {
                Some(NtHeaders::Headers32(_)) => self.real_addr32(*(ctx as *const WOW64_CONTEXT)) as u64,
                Some(NtHeaders::Headers64(_)) => self.real_addr64(*ctx),
                None => {
                    print_lg(LevelPrint::Error, "file is not loaded");
                    0
                }
            }
        }
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
pub struct Symbols {
    pub symbol_type: SymbolType,
    pub symbol_file: Vec<SymbolFile>,
}



pub static mut SYMBOLS_V: Lazy<Symbols> = Lazy::new(|| Symbols::default());
pub static mut IMAGE_BASE: u64 = 0;



pub fn load_symbol(linev: &[&str], line: &str) {
    unsafe {
        let psym = ptr::addr_of!(SYMBOLS_V);
        if (*ptr::addr_of!(ALL_ELM)).file.is_none() {
            print_lg(LevelPrint::ErrorO, "you must first specify a file");
            return;
        }
        if linev.len() > 1 && linev[0] == "s" || linev[0] == "symbol" {
            let path1 = line[linev[0].len()+1..].trim();
            let path = path1.replace("\"", "");
            if let Err(e) = pdb::from_pdb_file(&path) {
                print_lg(LevelPrint::ErrorO, format!("error for loading symbols from pdb file : {e}"));
            }else if (*psym).symbol_type == SymbolType::PDB{
                ALL_ELM.pdb_path = Some(path);
            }
        }else {
            if let Err(e) = dwarf::target_dwarf_info(&*pefile::section::SECTION_VS) {
                print_lg(LevelPrint::ErrorO, format!("Error target symbol dwarf: {e}"));
                return;
            }
            pdb::target_symbol();
        }
        
        if (*psym).symbol_type != SymbolType::Un {
            print_lg(LevelPrint::DebugO, format!("the symbol file was loaded with success\nsymbol type: {}", (*psym).symbol_type));
        } else {
            print_lg(LevelPrint::ErrorO, "the file does not contain a supported symbol format");
        }
    }
}


pub unsafe fn sym_init(h_proc: HANDLE) -> Result<(), anyhow::Error>{
    let symbol_pe = Dll::new("symbol_pe.dll")?;
    let sym_init: unsafe extern "C" fn(HANDLE, *const u8, u64) -> BOOL = mem::transmute(symbol_pe.get_func("sym_init")?);
    let pdb_path = (*&raw const ALL_ELM).pdb_path.clone().map(|f|f.as_ptr()).unwrap_or(ptr::null());
    let base_addr = if BASE_ADDR != 0 {BASE_ADDR} else {IMAGE_BASE};
    if sym_init(h_proc, pdb_path, base_addr) == 0 {
        return Err(anyhow!("failed to init symbol : {}", io::Error::last_os_error()));
    }
    Ok(())
}
