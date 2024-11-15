use crate::dbg::dbg_cmd::usages;
use crate::dbg::{RealAddr, BASE_ADDR};
use crate::pefile::function::{Find, FUNC_INFO};
use crate::pefile::{NtHeaders, NT_HEADER};
use crate::symbol::{SymbolFile, SYMBOLS_V};
use iced_x86::{Decoder, DecoderOptions, Instruction, Mnemonic};
use std::{io, mem, ptr};
use winapi::shared::minwindef::LPVOID;
use winapi::shared::ntdef::HANDLE;
use winapi::um::memoryapi::{ReadProcessMemory, VirtualProtectEx, VirtualQueryEx};
use winapi::um::winnt::{
    CONTEXT, MEMORY_BASIC_INFORMATION, PAGE_EXECUTE_READWRITE, WOW64_CONTEXT,
};
use crate::dbg::memory::deref_mem;
use crate::ut::{get_addr_va, get_addr_va32};
use crate::ut::cast::str_to;
use crate::ut::fmt::*;

pub struct JAddr {
    addr: u64,
    num_target: usize,
}

fn finder(sym: &SymbolFile, target: u64) -> bool {
    if sym.offset < 0 {
        false
    } else {
        sym.offset as u64 + unsafe { BASE_ADDR } == target
    }
}


pub fn is_valid_addr(h_proc: HANDLE, addr: u64) -> bool {
    unsafe {
        let mut mem_info = mem::zeroed();
        if VirtualQueryEx(h_proc, addr as LPVOID, &mut mem_info, size_of::<MEMORY_BASIC_INFORMATION>()) == 0 {
            false
        }else {
            true
        }
    }
}

pub fn handle_disasm(linev: &[&str], h_proc: HANDLE, ctx: *const CONTEXT) {
    if linev.len() < 2 {
        println!("{}", usages::USAGE_DISASM);
        return;
    }

    let addr_str = linev[1];
    let count_str = linev.get(2);
    let addr = match unsafe { NT_HEADER }.unwrap() {
        NtHeaders::Headers32(_) => unsafe {
            match get_addr_va32(addr_str, *(ctx as *const WOW64_CONTEXT)) {
                Ok(addr) => Ok(addr as u64),
                Err(e) => Err(e),
            }
        } 
        NtHeaders::Headers64(_) => unsafe { get_addr_va(addr_str, *ctx) }
    };
    match addr {
        Ok(addr) => disasm(h_proc, addr, count_str, ctx),
        Err(e) => {
            print_lg(LevelPrint::ErrorO, e);
            return;
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
                size = 2093;
            }
        } else {
            let mut mem_info = mem::zeroed();
            if VirtualQueryEx(h_proc, addr as LPVOID, &mut mem_info, size_of::<MEMORY_BASIC_INFORMATION>()) == 0 {
                print_lg(LevelPrint::ErrorO, format!("Failed to get the size of region of {:#x} : {}", addr, io::Error::last_os_error()));
                return;
            }
            size = mem_info.BaseAddress as usize +  mem_info.RegionSize - addr as usize;
        }
    }

    let mut buffer = vec![0u8; size];
    let mut old_protect: u32 = 0;
    unsafe {
        if VirtualProtectEx(h_proc, addr as LPVOID, buffer.len(), PAGE_EXECUTE_READWRITE, &mut old_protect) == 0 {
            print_lg(LevelPrint::ErrorO, format!("Failed to remove memory protection at address {:#x}: {}", addr, io::Error::last_os_error()));
            return;
        }
        if ReadProcessMemory(h_proc, addr as LPVOID, buffer.as_mut_ptr() as LPVOID, buffer.len(), ptr::null_mut()) == 0 {
            print_lg(LevelPrint::ErrorO, format!("Failed to read process memory: {}", io::Error::last_os_error()));
            return;
        }
        if VirtualProtectEx(h_proc, addr as LPVOID, buffer.len(), old_protect, &mut old_protect) == 0 {
            print_lg(LevelPrint::ErrorO, format!("Failed to restore memory protection at address {:#x}: {}", addr, io::Error::last_os_error()));
            return;
        }
    }
    let mut decoder = Decoder::with_ip(unsafe { NT_HEADER }.unwrap().get_bitness() as u32, &buffer, addr, DecoderOptions::NONE);
    let mut insn = Instruction::default();
    let mut i = 0;
    let j_jump = first_it(Decoder::with_ip(unsafe { NT_HEADER }.unwrap().get_bitness() as u32, &buffer, addr, DecoderOptions::NONE), count);
    let mut last_insn_is_ret = false;
    println!("\x1b[1m{: <16} {: <48} {: <32}{RESET_COLOR}", "Address", "Bytes", "Instruction");

    while decoder.can_decode() && i < count {
        decoder.decode_out(&mut insn);
        let start_index = (insn.ip() - addr) as usize;
        let instr_bytes = &buffer[start_index..start_index + insn.len()];
        let byte_str = instr_bytes.iter().map(|b| format!("{:02X}", b)).collect::<Vec<String>>().join(" ");
        let mut insns = insn.to_string();
        if last_insn_is_ret || j_jump.iter().any(|j| j.addr == insn.ip()) {
            println!("{ADDR_COLOR}{:#x}: ", insn.ip());
            println!("{ADDR_COLOR}{:#x}: ", insn.ip());
            println!("{ADDR_COLOR}{:#x}: {:<40}{}{RESET_COLOR}", insn.ip(), "", format!("label_{:x}:", insn.ip()));
            last_insn_is_ret = false;
        }
        if insn.near_branch_target() != 0 {
            unsafe {
                if let Some(sym) = (*&raw const SYMBOLS_V).symbol_file.iter().find(|&s| finder(s, insn.near_branch_target())) {
                    insns = insns.replace(&format!("{:016X}h", insn.near_branch_target()), &format!("{MAGENTA}{}{RESET_COLOR}", sym.name))
                } else {
                    insns = insns.replace(&format!("{:016X}h", insn.near_branch_target()), &format!("{MAGENTA}label_{:x}{RESET_COLOR}", insn.near_branch_target()));
                }
            }
        }

        if insn.is_ip_rel_memory_operand() {
            unsafe {
                let ptr_ip = insn.ip_rel_memory_address();
                if let Some(sym) = (*ptr::addr_of!(SYMBOLS_V)).symbol_file.iter().find(|&s| s.in_sym(ptr_ip, ctx)) {
                    let offset = ptr_ip - sym.real_addr(ctx);
                    insns.push_str(&format!(" ; {MAGENTA}{}<{:+#x}>{RESET_COLOR}", sym.name, offset));
                }else if let Some(f) = (*ptr::addr_of_mut!(FUNC_INFO)).iter().find(|f|f.find_in_func(ptr_ip)) {
                    let offset = ptr_ip - (f.BeginAddress as u64 + BASE_ADDR);
                    insns.push_str(&format!(" ; {MAGENTA}func_{:+#x}<{:+}>{RESET_COLOR}", f.BeginAddress, offset));
                }
                else {
                    if insn.memory_size().size() == NT_HEADER.unwrap().get_size_of_arch() {
                        let size = NT_HEADER.unwrap().get_size_of_arch();
                        let mut res = insn.ip_rel_memory_address();
                        let mut def_addr = Vec::new();
                        insns.push_str(&format!(" ; {}", CYAN_COLOR));
                        loop {
                            let mut new_addr = 0u64;
                            if ReadProcessMemory(h_proc, res as LPVOID, ptr::addr_of_mut!(new_addr) as LPVOID, size, &mut 0) == 0 {
                                insns.push_str(&format!("failed to read memory at address {:#x}", res));
                                break;
                            }
                            if is_valid_addr(h_proc, new_addr) {
                                insns.push_str(&format!(" -> {:#x}", res));
                                def_addr.push(res);
                                res = new_addr;
                            }else {
                                if let Some(sym) = (*ptr::addr_of_mut!(SYMBOLS_V)).symbol_file.iter().find(|s|s.in_sym(res, ctx)) {
                                    let offset = res - sym.real_addr(ctx);
                                    insns.push_str(&format!(" -> {MAGENTA}{}<{:#x}>{RESET_COLOR}", sym.name, offset));
                                } 
                                else if let Some(f) = (*ptr::addr_of_mut!(FUNC_INFO)).iter().find(|f|f.find_in_func(res)) {
                                    let offset = res - (f.BeginAddress as u64 + BASE_ADDR);
                                    insns.push_str(&format!(" -> func_{:#x}<{:+#x}>", f.BeginAddress, offset))
                                }
                                else {
                                    let mut buf = [0u8;260];
                                    if ReadProcessMemory(h_proc, res as LPVOID, buf.as_mut_ptr() as LPVOID, 260, &mut 0) == 0 {
                                        insns.push_str(&format!("failed to read memory at address {:#x}", res));
                                        break;
                                    }
                                    let mut nw_buf = Vec::new();
                                    for b in buf {
                                        if b != 0 {
                                            nw_buf.push(b);
                                        } else {
                                            break
                                        }
                                    }
                                    insns.push_str(&format!(" -> \"{}\"", deref_mem::espc(&nw_buf)));
                                }
                                break
                            }
                        }
                    }else if insn.memory_size().size() != 0 {
                        let mut value = 0u128;
                        if ReadProcessMemory(h_proc, ptr_ip as LPVOID, ptr::addr_of_mut!(value) as LPVOID, insn.memory_size().size(), &mut 0) == 0 {
                            insns.push_str(&format!(" ; failed to read memory at address {:#x} : {}", ptr_ip, io::Error::last_os_error()));
                        }else {
                            insns.push_str(&format!(" ; {INSTR_COLOR}{:#x}{RESET_COLOR}", value))
                        }
                    }
                }
            }
        }
        if insn.immediate64() != 0 {
            unsafe {
                if let Some(sym) = (*&raw mut SYMBOLS_V).symbol_file.iter().find(|&s| finder(s, insn.immediate64())) {
                    insns = insns.replace(&format!("{:X}h", insn.immediate64()), &sym.name);
                }
            }
        }
        println!("{ADDR_COLOR}{:#x}: {BYTES_COLOR}{:<48} {VALUE_COLOR}{insns}{RESET_COLOR}", insn.ip(), byte_str);
        if insn.mnemonic() == Mnemonic::Ret {
            last_insn_is_ret = true;
        }
        i += 1;
    }
}

pub fn first_it(decoder: Decoder, count: usize) -> Vec<JAddr> {
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
        i += 1
    }
    result
}
