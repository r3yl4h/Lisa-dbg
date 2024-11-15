use crate::dbg::dbg_cmd::usages::USAGE_FIND;
use std::{io, mem};
use winapi::shared::minwindef::LPVOID;
use winapi::shared::ntdef::HANDLE;
use winapi::um::memoryapi::{ReadProcessMemory, VirtualQueryEx};
use winapi::um::winnt::MEMORY_BASIC_INFORMATION;
use crate::ut::cast::{str_to, Char, ToType};
use crate::ut::fmt::{print_lg, LevelPrint};

pub fn handle_find(linev: &[&str], h_proc: HANDLE) {
    if linev.len() < 5 {
        print_lg(LevelPrint::DebugO, USAGE_FIND);
        return;
    }
    let (begin_addr, end_addr) = match (str_to::<u64>(linev[2]), str_to::<u64>(linev[3])) {
        (Ok(addr1), Ok(addr2)) => (addr1, addr2),
        (Err(e1), Ok(_)) => {
            print_lg(LevelPrint::ErrorO, format!("failed to parse begin addr: {e1}"));
            return;
        }
        (Ok(_), Err(e2)) => {
            print_lg(LevelPrint::ErrorO, format!("failed to parse end addr: {e2}"));
            return;
        }
        (Err(e1), Err(e2)) => {
            print_lg(LevelPrint::ErrorO, format!("failed to parse begin & end addr, 1: {e1}, 2: {e2}"));
            return;
        }
    };

    match linev[1] {
        "uint8_t" | "u8" | "byte" => find_seq_byte::<u8>(h_proc, begin_addr, end_addr, linev[4..].join(" ")),
        "i8" | "int8_t" => find_seq_byte::<i8>(h_proc, begin_addr, end_addr, linev[4..].join(" ")),
        "char" => find_seq_byte::<Char>(h_proc, begin_addr, end_addr, linev[4..].join(" ")),
        "uint16_t" | "u16" | "word" => find_seq_byte::<u16>(h_proc, begin_addr, end_addr, linev[4..].join(" ")),
        "int16_t" | "i16" | "short" => find_seq_byte::<i16>(h_proc, begin_addr, end_addr, linev[4..].join(" ")),
        "uint32_t" | "u32" | "dword" => find_seq_byte::<u32>(h_proc, begin_addr, end_addr, linev[4..].join(" ")),
        "int32_t" | "i32" | "int" | "long" => find_seq_byte::<i32>(h_proc, begin_addr, end_addr, linev[4..].join(" ")),
        "uint64_t" | "u64" | "qword" => find_seq_byte::<u64>(h_proc, begin_addr, end_addr, linev[4..].join(" ")),
        "int64_t" | "i64" | "long long" => find_seq_byte::<i64>(h_proc, begin_addr, end_addr, linev[4..].join(" ")),
        _ => print_lg(LevelPrint::ErrorO, format!("unknown type: {}", linev[1])),
    }
}

fn find_seq_byte<T: ToType + Default + Clone + std::fmt::Debug + PartialEq + std::fmt::Display>(
    h_proc: HANDLE,
    beg_addr: u64,
    end_addr: u64,
    args: String,
) {
    let wordv = args.split(",").map(|s| s.trim()).collect::<Vec<&str>>();
    let mut result = Vec::new();

    for word in wordv {
        if word.starts_with("\"") || word.starts_with("'") {
            let new_word = word[1..word.len() - 1].trim();
            for c in new_word.chars() {
                result.push(T::from_char(c));
            }
        } else {
            match T::from_str_value(word) {
                Ok(val) => result.push(val),
                Err(e) => print_lg(LevelPrint::ErrorO, format!("failed to parse value: {e}")),
            }
        }
    }

    let size_mem = if end_addr != 0 {
        if beg_addr < end_addr {
            (end_addr - beg_addr) as usize
        } else {
            print_lg(LevelPrint::ErrorO, "you cannot specify a start address greater than the end address".to_string());
            return;
        }
    } else {
        unsafe {
            let mut mem_info = mem::zeroed();
            if VirtualQueryEx(h_proc, beg_addr as LPVOID, &mut mem_info, size_of::<MEMORY_BASIC_INFORMATION>()) == 0 {
                print_lg(LevelPrint::ErrorO, format!("failed to query memory info for addr {:#x}: {}", beg_addr, io::Error::last_os_error()));
                return;
            }
            mem_info.RegionSize
        }
    };

    if size_of::<T>() > size_mem {
        print_lg(LevelPrint::ErrorO, format!("the size of the memory to analyze is smaller than a single {}", std::any::type_name::<T>()));
        return;
    }

    let mut plage_mem = vec![T::default(); size_mem / size_of::<T>()];
    unsafe {
        let mut bytes_read = 0;
        if ReadProcessMemory(h_proc, beg_addr as LPVOID, plage_mem.as_mut_ptr() as LPVOID, size_mem, &mut bytes_read) == 0 {
            print_lg(LevelPrint::ErrorO, format!("failed to read memory {size_mem} bytes at address {:#x}: {}", beg_addr, io::Error::last_os_error()));
            return;
        }

        if bytes_read == 0 {
            print_lg(LevelPrint::ErrorO, "No bytes read from memory".to_string());
            return;
        }
    }

    if result.len() > plage_mem.len() {
        print_lg(LevelPrint::ErrorO, "you specified too many elements for the elements".to_string());
        return;
    }

    let mut found_addr = Vec::new();
    for i in 0..plage_mem.len() {
        if plage_mem[i] == result[0] && plage_mem[i..].len() >= result.len() {
            if plage_mem[i..i + result.len()] == *result {
                found_addr.push(beg_addr + (i * size_of::<T>()) as u64);
            }
        }
    }

    if found_addr.is_empty() {
        print_lg(LevelPrint::ErrorO, format!("element {:?} not found", result));
    } else {
        for addr in found_addr {
            print_lg(LevelPrint::DebugO, format!("element found at address {:#x}", addr));
        }
    }
}
