use crate::dbg::dbg_cmd::x32;
use crate::dbg::dbg_cmd::x64::modifier;
use crate::dbg::memory::set;
use crate::usage;
use winapi::um::winnt::{CONTEXT, HANDLE, WOW64_CONTEXT};
use crate::pefile::{NtHeaders, NT_HEADER};
use crate::ut::fmt::{print_lg, LevelPrint};

pub fn set_element(h_proc: HANDLE, ctx: *mut CONTEXT, linev: &[&str]) {
    if linev.len() < 3 {
        println!("{}", usage::USAGE_SET);
        return;
    }
    let type_set = linev[1].to_lowercase();
    let target = &linev[2..];
    match type_set.as_str() {
        "memory" | "mem" => set::set_memory::set_memory(h_proc, ctx, target),
        "mem-protect" | "memory-protect" => set::set_protect::change_protect(h_proc, ctx, target),
        "register" | "reg" => {
            unsafe {
                match NT_HEADER.unwrap() {
                    NtHeaders::Headers32(_) => x32::modifier32::register::set_register32(&target, &mut *(ctx as *mut WOW64_CONTEXT)),
                    NtHeaders::Headers64(_) => modifier::register::set_register64(target, &mut *ctx),
                }
            }
        },
        _ => print_lg(LevelPrint::ErrorO, format!("unknow element {}", linev[1])),
    }
}
