use crate::dbg::dbg_cmd::usages;
use crate::dbg::{BASE_ADDR};
use crate::pefile::function::FUNC_INFO;
use crate::pefile::{get_section_of_rva, NtHeaders, NT_HEADER};
use crate::symbol::{SymbolFile, SYMBOLS_V};
use iced_x86::{Decoder, DecoderOptions, Formatter, Instruction, IntelFormatter, Mnemonic, SymbolResolver, SymbolResult};
use std::{io, mem, ptr};
use anyhow::anyhow;
use winapi::shared::minwindef::LPVOID;
use winapi::shared::ntdef::HANDLE;
use winapi::um::memoryapi::VirtualQueryEx;
use winapi::um::winnt::{CONTEXT, MEMORY_BASIC_INFORMATION, MEM_COMMIT, RUNTIME_FUNCTION};
use crate::dbg::memory::deref_mem;
use crate::ut::{get_addr_br, get_addr_va};
use crate::ut::cast::str_to;
use crate::ut::fmt::*;
use crate::ut::mem::read_mem;

#[derive(Clone)]
pub struct JAddr {
    addr: u64,
    num_target: usize,
}

#[derive(Clone)]
struct Sym {
    pub sym_file: Vec<SymbolFile>,
    pub func: Vec<RUNTIME_FUNCTION>,
    pub j_jump: Vec<JAddr>,
    pub ctx: *const CONTEXT,
}

impl Sym {
    pub fn from_va_sym(sym_file: &[SymbolFile], func: &[RUNTIME_FUNCTION], j_jump: Vec<JAddr>, ctx: *const CONTEXT) -> Sym {
        Sym {sym_file: sym_file.to_vec(), func: func.to_vec(), ctx, j_jump}
    }
}


impl SymbolResolver for Sym {
    fn symbol(&mut self, _insn: &Instruction, _op: u32, _insn_op: Option<u32>, rel_addr: u64, _addr_size: u32) -> Option<SymbolResult<'_>> {
        if let Some(sym) = self.sym_file.iter().find(|s|s.addr_ot(self.ctx) == rel_addr) {
            Some(SymbolResult::with_string(rel_addr, format!("{BYTES_COLOR}{}{RESET_COLOR}", sym.name)))
        }else if let Some(f) = self.func.iter().find(|s| s.BeginAddress as u64 + unsafe {BASE_ADDR} == rel_addr) {
            return Some(SymbolResult::with_string(rel_addr, format!("{BYTES_COLOR}func_{:x}{RESET_COLOR}", f.BeginAddress)))
        }else if let Some(_) = self.j_jump.iter().find(|j|j.addr == rel_addr) {
            return Some(SymbolResult::with_string(rel_addr, format!("{BYTES_COLOR}label_{:x}{RESET_COLOR}", rel_addr)))
        } else {
            None
        }
    }
}


/*fn finder(sym: &SymbolFile, target: u64) -> bool {
    if sym.offset < 0 {
        false
    } else {
        sym.offset as u64 + unsafe { BASE_ADDR } == target
    }
}*/


pub fn is_valid_addr(h_proc: HANDLE, addr: u64) -> bool {
    if h_proc.is_null() {
        get_section_of_rva(addr).is_some()
    }
    else {
        unsafe {
            let mut mem_info = mem::zeroed();
            if VirtualQueryEx(h_proc, addr as LPVOID, &mut mem_info, size_of::<MEMORY_BASIC_INFORMATION>()) == 0 {
                false
            } else {
                mem_info.State == MEM_COMMIT
            }
        }
    }
}




pub fn get_addr_va1(addr_str: &str, ctx: *const CONTEXT) -> Result<u64, anyhow::Error> {
    if ctx.is_null() {
        return get_addr_br(addr_str);
    }
    match unsafe { NT_HEADER } {
        Some(NtHeaders::Headers32(_)) | Some(NtHeaders::Headers64(_)) => get_addr_va(addr_str, ctx),
        None => Err(anyhow!("you must load a file for disasm")),
    }
}


pub fn handle_disasm(linev: &[&str], h_proc: HANDLE, ctx: *const CONTEXT) {
    if linev.len() < 2 {
        println!("{}", usages::USAGE_DISASM);
        return;
    }

    let addr_str = linev[1];
    let count_str = linev.get(2);
    match get_addr_va1(addr_str, ctx) {
        Ok(addr) => disasm(h_proc, addr, count_str, ctx),
        Err(e) => {
            print_lg(LevelPrint::ErrorO, e);
            return;
        }
    }
}


fn get_memory_info(h_proc: HANDLE, addr: u64) -> Result<MEMORY_BASIC_INFORMATION, anyhow::Error> {
    unsafe {
        if h_proc.is_null() {
            return if let Some(sec) = get_section_of_rva(addr) {
                let mut mem_info: MEMORY_BASIC_INFORMATION = mem::zeroed();
                mem_info.RegionSize = sec.content.len();
                mem_info.BaseAddress = sec.addr as LPVOID;
                Ok(mem_info)
            } else {
                Err(anyhow!("invalid address"))
            }
        }

        let mut mem_info = mem::zeroed();
        if VirtualQueryEx(h_proc, addr as LPVOID, &mut mem_info, size_of::<MEMORY_BASIC_INFORMATION>()) == 0 {
            Err(anyhow!("Failed to get the size of region of {:#x} : {}", addr, io::Error::last_os_error()))
        } else {
            Ok(mem_info)
        }
    }
}



fn disasm(h_proc: HANDLE, addr: u64, count_str: Option<&&str>, ctx: *const CONTEXT) {
    let count = if count_str.is_some() {
        match str_to::<usize>(count_str.unwrap()) {
            Ok(count) => count,
            Err(e) => {
                print_lg(LevelPrint::ErrorO, format!("Invalid count : {e}"));
                return;
            }
        }
    } else {
        usize::MAX
    };
    let size;

    unsafe {
        if (*&raw const FUNC_INFO).len() != 0 {
            if let Some(func) = (*&raw const FUNC_INFO).iter().find(|f| {
                f.BeginAddress as u64 + BASE_ADDR <= addr && f.EndAddress as u64 + BASE_ADDR > addr
            }) {
                size = (func.EndAddress as u64 + BASE_ADDR - addr) as usize;
            } else {
                match get_memory_info(h_proc, addr) {
                    Ok(mem_info) => size = mem_info.BaseAddress as usize +  mem_info.RegionSize - addr as usize,
                    Err(e) => {
                        print_lg(LevelPrint::ErrorO, e);
                        return;
                    },
                    
                }
            }
        } else {
            match get_memory_info(h_proc, addr) {
                Ok(mem_info) => size = mem_info.BaseAddress as usize + mem_info.RegionSize - addr as usize,
                Err(e) => {
                    print_lg(LevelPrint::ErrorO, e);
                    return;
                },
            }
        }
    }

    let mut buffer = vec![0u8; size];
    if let Err(e) = read_mem(h_proc, addr, &mut buffer) {
        print_lg(LevelPrint::ErrorO, e);
        return;
    }
    let (insnv, j_jump) = first_it(Decoder::with_ip(unsafe { NT_HEADER }.unwrap().get_bitness() as u32, &buffer, addr, DecoderOptions::NONE), count);

    let sym = unsafe { Sym::from_va_sym(&*ptr::addr_of!(SYMBOLS_V.symbol_file) , &*ptr::addr_of_mut!(FUNC_INFO), j_jump, ctx)};

    let mut fmter = IntelFormatter::with_options(Some(Box::new(sym.clone())), None);
    let mut out = String::new();
    
    
    println!("\x1b[1m{: <16} {: <48} {: <32}{RESET_COLOR}", "Address", "Bytes", "Instruction");
    for insn in insnv {
        out.clear();
        fmter.format(&insn, &mut out);
        if let Some(sym) = sym.sym_file.iter().find(|s|s.addr_ot(ctx) == insn.ip()) {
            println!("{ADDR_COLOR}{:#x}: ", insn.ip());
            println!("{ADDR_COLOR}{:#x}: ", insn.ip());
            println!("{ADDR_COLOR}{:#x}: {:<40}{}:{RESET_COLOR}", insn.ip(), "", sym.name);
        }
        else if let Some(f) = sym.func.iter().find(|f|f.BeginAddress as u64 + unsafe {BASE_ADDR} == insn.ip()) {
            println!("{ADDR_COLOR}{:#x}: ", insn.ip());
            println!("{ADDR_COLOR}{:#x}: ", insn.ip());
            println!("{ADDR_COLOR}{:#x}: {:<40}{}{RESET_COLOR}", insn.ip(), "", format!("func_{:x}:", f.BeginAddress));
        }
        else if let Some(_) = sym.j_jump.iter().find(|j|j.addr == insn.ip()) {
            println!("{ADDR_COLOR}{:#x}: ", insn.ip());
            println!("{ADDR_COLOR}{:#x}: ", insn.ip());
            println!("{ADDR_COLOR}{:#x}: {:<40}{}{RESET_COLOR}", insn.ip(), "", format!("label_{:x}:", insn.ip()));
        }
        if insn.is_ip_rel_memory_operand() {
            let mut r1 = insn.ip_rel_memory_address();
            let mem_size = insn.memory_displ_size() as usize;
            out.push_str(&format!("{CYAN_COLOR} ;"));
            unsafe {
                if mem_size == NT_HEADER.unwrap().get_size_of_arch() {
                    let mut addrpass = Vec::new();
                    let mut new_v = 0u64;
                    out.push_str(&format!(" {r1:#x}"));
                    loop {
                        if let Err(e) = read_mem(h_proc, r1, std::slice::from_raw_parts_mut(ptr::addr_of_mut!(new_v) as *mut u8, mem_size)) {
                            print_lg(LevelPrint::ErrorO, format!("<{insn}> - Failed to read process memory at address {:#x} : {e}", r1));
                            break;
                        }

                        if is_valid_addr(h_proc, new_v) {
                            if addrpass.contains(&new_v) {
                                out.push_str(&format!(" <-> {:#x}", new_v));
                                break
                            }
                            addrpass.push(new_v);
                            out.push_str(&format!(" -> {:#x}", new_v));
                            r1 = new_v;
                        }else {
                            if let Some(s) = sym.sym_file.iter().find(|s|s.addr_ot(ctx) == r1) {
                                out.push_str(&format!(" -> {ADDR_COLOR}{}{RESET_COLOR}", s.name));
                            }

                            else if let Some(f) = sym.func.iter().find(|f|f.BeginAddress as u64 + BASE_ADDR == r1) {
                                out.push_str(&format!(" -> {BYTES_COLOR}func_{:x}{RESET_COLOR}", f.BeginAddress as u64))
                            }

                            else if let Some(j) = sym.j_jump.iter().find(|j|j.addr == r1) {
                                out.push_str(&format!(" -> {BYTES_COLOR}label_{:x}{RESET_COLOR}", j.addr));
                            }
                            else {
                                match read_str(h_proc, r1, insn) {
                                    Ok(str1) => out.push_str(&format!(" -> {str1}")),
                                    Err(e) => print_lg(LevelPrint::ErrorO, e),
                                }
                            }
                            break;
                        }
                    }
                }
                else if mem_size != 0 {
                    let mut u128v = 0u128;
                    if let Err(e) = read_mem(h_proc, r1, std::slice::from_raw_parts_mut(ptr::addr_of_mut!(u128v) as *mut u8, mem_size)) {
                        print_lg(LevelPrint::ErrorO, format!("<{insn}> - Failed to read process memory at address {:#x} : {e}", r1));
                        continue;
                    }
                    out.push_str(&format!("{:#x}", u128v));
                }else {
                    match read_str(h_proc, r1, insn) {
                        Ok(str1) => out.push_str(&format!(" {str1}")),
                        Err(e) => {
                            print_lg(LevelPrint::ErrorO, e);
                            continue;
                        }
                    }
                }
            }
        }
        let start_index = (insn.ip() - addr) as usize;
        let instr_bytes = &buffer[start_index..start_index + insn.len()];
        let byte_str = instr_bytes.iter().map(|b| format!("{:02X}", b)).collect::<Vec<String>>().join(" ");
        println!("{ADDR_COLOR}{:#x}: {}{:<48} {VALUE_COLOR}{out}{RESET_COLOR}", insn.ip(), "\x1b[38;5;166m", byte_str);
    }
}



pub fn read_str(h_proc: HANDLE, addr: u64, insn: Instruction) -> Result<String, anyhow::Error> {
    let mut str_buf = [0u8;260];
    if let Err(e) = read_mem(h_proc, addr, &mut str_buf) {
        return Err(anyhow!("<{insn}> - Failed to read process memory at address {:#x} : {}", addr, e));
    }
    let f: Vec<u8> = str_buf.iter().take_while(|&&c| c != 0).map(|&c| c).collect();
    Ok(format!("\"{}\"", deref_mem::espc(&f)))
}



pub fn first_it(decoder: Decoder, count: usize) -> (Vec<Instruction>, Vec<JAddr>) {
    let mut res_insn = Vec::new();
    let mut decoder = decoder;
    let mut insn = Instruction::default();
    let mut i = 0;
    let mut result: Vec<JAddr> = Vec::new();
    while decoder.can_decode() && i < count {
        decoder.decode_out(&mut insn);
        if insn.near_branch_target() != 0 {
            if let Some(jmp) = result.iter_mut().find(|f| f.addr == insn.near_branch_target()) {
                jmp.num_target += 1
            } else {
                result.push(JAddr {
                    addr: insn.near_branch_target(),
                    num_target: 0,
                });
            }
        }
        if insn.mnemonic() == Mnemonic::Ret {
            result.push(JAddr {
                addr: insn.next_ip(),
                num_target: 0,
            })
        }
        res_insn.push(insn);
        i += 1
    }
    (res_insn, result)
}
