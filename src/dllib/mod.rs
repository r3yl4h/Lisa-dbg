use std::{io};
use anyhow::anyhow;
use winapi::shared::minwindef::{FARPROC, HMODULE};
use winapi::um::libloaderapi::{FreeLibrary, GetProcAddress, LoadLibraryA};

pub struct Dll {
    hmod: HMODULE,
}

impl Dll {
    pub fn new(name: &str) -> Result<Dll, anyhow::Error> {
        unsafe {
            let hmod = LoadLibraryA(format!("{name}\0").as_ptr() as *const i8);
            if hmod.is_null() {
                return Err(anyhow!("Failed to load {name} : {}", io::Error::last_os_error()));
            }
            Ok(Dll { hmod })
        }
    }

    pub fn get_func(&self, name: &str) -> Result<FARPROC, anyhow::Error>{
        unsafe {
            let addr_func = GetProcAddress(self.hmod, format!("{name}\0").as_ptr() as *const i8);
            if addr_func.is_null() {
                return Err(anyhow!("Failed to get {name} function : {}",io::Error::last_os_error()));
            }
            Ok(addr_func)
        }
    }
}



impl Drop for Dll {
    fn drop(&mut self) {
        unsafe {
            FreeLibrary(self.hmod);
        }
    }
}