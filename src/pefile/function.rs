use crate::pefile;
use std::{ptr, slice};
use winapi::um::winnt::{IMAGE_DATA_DIRECTORY, RUNTIME_FUNCTION};
use crate::dbg::BASE_ADDR;
use crate::ut::fmt::{print_lg, LevelPrint};

pub static mut FUNC_INFO: Vec<RUNTIME_FUNCTION> = Vec::new();


pub trait Find {
    fn find_in_func(&self, addr: u64) -> bool;
}

impl Find for RUNTIME_FUNCTION {
    fn find_in_func(&self, addr: u64) -> bool {
        unsafe {
            let start = self.BeginAddress as u64 + BASE_ADDR;
            let end = self.EndAddress as u64 + BASE_ADDR;
            addr >= start && addr <= end
        }
    }
}

pub fn parse_pdata(pdata_dir: IMAGE_DATA_DIRECTORY) {
    if pdata_dir.VirtualAddress == 0 || pdata_dir.Size == 0 {
        print_lg(LevelPrint::WarningO, "no section is IMAGE_DIRECTORY_ENTRY_EXCEPTION");
        return;
    }
    let rva_pdata = pdata_dir.VirtualAddress;
    for section in unsafe { &*pefile::section::SECTION_VS } {
        if section.addr <= rva_pdata && section.addr + section.content.len() as u32 >= pdata_dir.VirtualAddress + pdata_dir.Size {
            let runt_size = section.content.len() / size_of::<RUNTIME_FUNCTION>();
            let base_pdata = section.content.as_ptr() as *const RUNTIME_FUNCTION;
            let mut runt_func = unsafe { slice::from_raw_parts(base_pdata, runt_size) }.to_vec();
            runt_func.retain(|f| f.BeginAddress != 0);
            unsafe {
                let pfunc_info = ptr::addr_of_mut!(FUNC_INFO);
                (*pfunc_info).clear();
                (*pfunc_info).extend_from_slice(&runt_func);
            }
            return;
        }
    }
    print_lg(LevelPrint::WarningO, "no section is IMAGE_DIRECTORY_ENTRY_EXCEPTION");
}
