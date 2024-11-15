use crate::dbg::dbg_cmd::usages;
use crate::dbg::dbg_cmd::x32::info_reg::ToValue32;
use crate::dbg::memory::deref_mem;
use winapi::shared::ntdef::HANDLE;
use winapi::um::winnt::WOW64_CONTEXT;
use crate::ut::cast::str_to;
use crate::ut::fmt::{print_lg, LevelPrint};

pub fn handle_deref32(linev: &[&str], ctx: WOW64_CONTEXT, h_proc: HANDLE) {
    if linev.len() < 3 {
        eprintln!("{}", usages::USAGE_DEREF);
        return;
    }
    let dtype = linev[1];
    let target = linev[2];
    let address = if let Ok(addr) = str_to::<u32>(target) {
        addr
    } else {
        ctx.str_to_ctx(target)
    };

    if address == 0 {
        print_lg(LevelPrint::ErrorO, "invalid register or null address");
        return;
    }
    if let Err(err) = deref_mem::deref_memory(h_proc, dtype, address as usize) {
        print_lg(LevelPrint::ErrorO, err);
    }
}
