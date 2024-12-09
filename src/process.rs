use std::ffi::{c_char, CStr};
use std::{io, mem};
use anyhow::anyhow;
use winapi::um::handleapi::CloseHandle;
use winapi::um::processthreadsapi::GetProcessId;
use winapi::um::tlhelp32::{CreateToolhelp32Snapshot, Module32First, Module32Next, Process32First, Process32Next, MODULEENTRY32, PROCESSENTRY32, TH32CS_SNAPMODULE, TH32CS_SNAPPROCESS};
use winapi::um::winnt::HANDLE;

pub fn get_pid_with_name(name: &str) -> Result<u32, String> {
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if snapshot.is_null() {
        return Err(format!("failed to create tool snashot : {}", io::Error::last_os_error()));
    }
    let mut entry: PROCESSENTRY32 = unsafe { mem::zeroed() };
    entry.dwSize = size_of::<PROCESSENTRY32>() as u32;
    if unsafe { Process32First(snapshot, &mut entry) } == 0 {
        return Err(format!("failed to get first process entry : {}", io::Error::last_os_error()));
    }
    loop {
        let name_proc = unsafe { CStr::from_ptr(entry.szExeFile.as_ptr() as *const c_char) }.to_string_lossy();
        if name_proc == name {
            unsafe {
                CloseHandle(snapshot);
            }
            return Ok(entry.th32ProcessID);
        }
        if unsafe { Process32Next(snapshot, &mut entry) } == 0 {
            break;
        }
    }
    unsafe {
        CloseHandle(snapshot);
    }
    Err("Process not found".to_string())
}


pub fn get_module(h_proc: HANDLE) -> Result<Vec<MODULEENTRY32>, anyhow::Error> {
    if h_proc.is_null() {
        return Err(anyhow!("you must have started the process to be able to use this option"));
    }
    let mut result = Vec::new();
    unsafe {
        let pid = GetProcessId(h_proc);
        let mod_snap = CreateToolhelp32Snapshot(TH32CS_SNAPMODULE, pid);
        if mod_snap.is_null() {
            return Err(anyhow!("failed to create module : {}", io::Error::last_os_error()));
        }
        let mut entry32: MODULEENTRY32 = mem::zeroed();
        entry32.dwSize = size_of::<MODULEENTRY32>() as u32;
        if Module32First(mod_snap, &mut entry32) == 0 {
            return Err(anyhow!("Failed to get first module : {}", io::Error::last_os_error()));
        }
        loop {
            result.push(entry32);
            if Module32Next(mod_snap, &mut entry32) == 0 {
                break
            }
        }
        CloseHandle(mod_snap);
    }
    if result.len() == 0 {
        return Err(anyhow!("Module not found for this process"));
    }
    Ok(result)
}
