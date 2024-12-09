use crate::dbg::memory::{breakpoint, watchpoint};
use crate::dbg::*;
use std::os::raw::c_char;
use std::{io, mem, ptr};
use winapi::shared::minwindef::LPVOID;
use winapi::um::debugapi::{ContinueDebugEvent, WaitForDebugEventEx};
use winapi::um::fileapi::GetFinalPathNameByHandleA;
use winapi::um::handleapi::CloseHandle;
use winapi::um::memoryapi::VirtualQueryEx;
use winapi::um::minwinbase::*;
use winapi::um::processthreadsapi::{CreateProcessA, PROCESS_INFORMATION, STARTUPINFOA};
use winapi::um::winbase::{DEBUG_PROCESS, INFINITE};
use winapi::um::winnt::*;
use crate::cli::ALL_ELM;
use crate::pefile::export::get_export_func_in_dll;
use crate::symbol::{SymbolFile, SYMBOLS_V};
use crate::ut::fmt::{print_lg, LevelPrint};



struct DllLoad {
    pub dll_base: LPVOID,
    pub dll_name: String,
}


pub fn debug_loop(h_proc: HANDLE) {
    let mut dll_load = Vec::new();
    unsafe {
        let mut debug_event = mem::zeroed::<DEBUG_EVENT>();
        let mut c_dbg = DbgState::Continue;
        while c_dbg == DbgState::Continue {
            if WaitForDebugEventEx(&mut debug_event, INFINITE) == 0 {
                let mut cd_dbg = c_dbg;
                print_lg(LevelPrint::Critical1(debug_event.dwProcessId, &mut cd_dbg), format!("failed to WaitForDebugEventEx : {}", io::Error::last_os_error()));
                return;
            }
            match debug_event.dwDebugEventCode {
                EXCEPTION_DEBUG_EVENT => {
                    let except_addr = debug_event.u.Exception().ExceptionRecord.ExceptionAddress as u64;
                    match debug_event.u.Exception().ExceptionRecord.ExceptionCode {
                        EXCEPTION_BREAKPOINT | STATUS_WX86_BREAKPOINT => {
                            if let Some(after_b) = (*ptr::addr_of!(ALL_ELM)).after_b.iter().find(|a|a.after_b == except_addr) {
                                handle_point::handle_after_b(h_proc, *after_b, &mut c_dbg, debug_event);
                            }
                            if let Some(hook_func) = (*ptr::addr_of_mut!(ALL_ELM)).hook.iter().find(|a| a.target + BASE_ADDR == except_addr) {
                                handle_point::handle_hook_func(h_proc, *hook_func, debug_event, &mut c_dbg);
                            }else {
                                if let Some(b) = (*ptr::addr_of_mut!(ALL_ELM)).break_rva.iter().find(|s|s.addr + BASE_ADDR == except_addr) {
                                    breakpoint::handle_br(h_proc, debug_event, b.addr + BASE_ADDR, b.origin_b, &mut c_dbg);
                                }
                                if let Some(b) = (*ptr::addr_of_mut!(ALL_ELM)).break_va.iter().find(|s|s.addr == except_addr) {
                                    breakpoint::handle_br(h_proc, debug_event, b.addr, b.origin_b, &mut c_dbg);
                                }
                                if let Some(b) = (*ptr::addr_of_mut!(ALL_ELM)).break_ret.iter_mut().find(|v|v.addr + BASE_ADDR == except_addr) {
                                    breakpoint::set_breakpoint_in_ret_func(h_proc, debug_event, b);
                                }
                                if let Some(b) = (*ptr::addr_of_mut!(ALL_ELM)).break_ret_va.iter_mut().find(|v|v.addr == except_addr) {
                                    breakpoint::set_breakpoint_in_ret_func(h_proc, debug_event, b);
                                }
                            }
                        }
                        EXCEPTION_SINGLE_STEP | STATUS_WX86_SINGLE_STEP => 
                            handle_point::handle_single_step(debug_event, except_addr, h_proc, &mut c_dbg),
                        EXCEPTION_ARRAY_BOUNDS_EXCEEDED =>
                            print_lg(LevelPrint::Error, format!("The code tries to access an invalid index in the table : {:#x}", debug_event.u.Exception().ExceptionRecord.ExceptionAddress as u64)),

                        EXCEPTION_DATATYPE_MISALIGNMENT =>
                            print_lg(LevelPrint::Critical1(debug_event.dwProcessId, &mut c_dbg), format!("An alignment problem occurred at address {:#x} and the system does not provide alignment", except_addr)),

                        EXCEPTION_FLT_DENORMAL_OPERAND =>
                            print_lg(LevelPrint::Critical1(debug_event.dwProcessId, &mut c_dbg), format!("One of the operands of a floating point operation is too small to be considered a floating point at address {:#x}", except_addr)),

                        EXCEPTION_FLT_DIVIDE_BY_ZERO =>
                            print_lg(LevelPrint::Critical1(debug_event.dwProcessId, &mut c_dbg), format!("The thread attempted to divide a floating point value by a floating point divisor of zero at address {:#x}", except_addr)),

                        EXCEPTION_FLT_INEXACT_RESULT =>
                                print_lg(LevelPrint::Critical1(debug_event.dwProcessId, &mut c_dbg), format!("The result of a floating point operation cannot be represented exactly as a decimal fraction at address {:#x}", except_addr)),

                        EXCEPTION_FLT_INVALID_OPERATION =>
                            print_lg(LevelPrint::Critical1(debug_event.dwProcessId, &mut c_dbg), format!("An error with floating point numbers occurred at address {:#x}", except_addr)),

                        EXCEPTION_FLT_OVERFLOW =>
                            print_lg(LevelPrint::Critical1(debug_event.dwProcessId, &mut c_dbg), format!("A floating point operation resulted in a value too large to represent at address {:#x}", except_addr)),


                        EXCEPTION_ILLEGAL_INSTRUCTION =>
                            print_lg(LevelPrint::Critical1(debug_event.dwProcessId, &mut c_dbg), format!("bad instruction at address {:#x}", except_addr)),


                        EXCEPTION_STACK_OVERFLOW =>
                            print_lg(LevelPrint::Critical1(debug_event.dwProcessId, &mut c_dbg), format!("stack overflow at address {:#x}", except_addr)),

                        EXCEPTION_ACCESS_VIOLATION => {
                            let access_type = debug_event.u.Exception().ExceptionRecord.ExceptionInformation[0];
                            let drs = debug_event.u.Exception().ExceptionRecord.ExceptionInformation[1];
                            let access_str = match access_type {
                                0 => "read",
                                1 => "write",
                                8 => "execute",
                                _ => "unknown",
                            };
                            let mut mem_info: MEMORY_BASIC_INFORMATION = mem::zeroed();
                            let query_result = VirtualQueryEx(h_proc, drs as LPVOID, &mut mem_info, size_of::<MEMORY_BASIC_INFORMATION>());
                            if query_result == 0 {
                                print_lg(LevelPrint::Critical1(debug_event.dwProcessId, &mut c_dbg), format!("Failed to query memory information : {}", io::Error::last_os_error()));
                            } else {
                                print_lg(LevelPrint::Critical1(debug_event.dwProcessId, &mut c_dbg), format!("memory access violation for '{access_str}' at address {:#x} caused by instruction at address {:#x}", drs, except_addr));
                                memory::mem_info::print_mem_info(mem_info);
                            }
                        }
                        _ => {}
                    }
                }
                CREATE_PROCESS_DEBUG_EVENT => {
                    print_lg(LevelPrint::Debug, format!("Process created at address: {:#x}", debug_event.u.CreateProcessInfo().lpBaseOfImage as u64));
                    BASE_ADDR = debug_event.u.CreateProcessInfo().lpBaseOfImage as u64;
                    init(h_proc);
                    watchpoint::set_watchpoint(debug_event, h_proc);
                }
                EXIT_PROCESS_DEBUG_EVENT => {
                    print_lg(LevelPrint::Debug, format!("Process exited with exit code : {}", debug_event.u.ExitProcess().dwExitCode));
                    c_dbg = DbgState::NeedStop;
                }
                CREATE_THREAD_DEBUG_EVENT => print_lg(LevelPrint::Debug, format!("Thread created : {:#x}", debug_event.u.CreateThread().lpStartAddress.unwrap() as u64)),
                EXIT_THREAD_DEBUG_EVENT => print_lg(LevelPrint::Debug, format!("Thread exited with exit code : {}", debug_event.u.ExitThread().dwExitCode)),
                LOAD_DLL_DEBUG_EVENT => {
                    let dll_base = debug_event.u.LoadDll().lpBaseOfDll;
                    let h_file = debug_event.u.LoadDll().hFile;
                    let mut buffer: [c_char; winapi::shared::minwindef::MAX_PATH] = [0; winapi::shared::minwindef::MAX_PATH];
                    let len = GetFinalPathNameByHandleA(h_file, buffer.as_mut_ptr(), winapi::shared::minwindef::MAX_PATH as u32, 0);
                    if len > 0 {
                        let path = std::slice::from_raw_parts(buffer.as_ptr() as *const u8, len as usize);
                        if let Ok(cstr) = std::str::from_utf8(path) {
                            let display_path = if cstr.starts_with(r"\\?\") {
                                &cstr[4..]
                            } else {
                                cstr
                            };
                            dll_load.push(DllLoad { dll_base,  dll_name: display_path.to_string()});
                            print_lg(LevelPrint::Debug, format!("Dll at address : {:#x} has been loaded ;{}", dll_base as u64, display_path));
                        } else {
                            print_lg(LevelPrint::Debug, format!("Dll at address : {:#x} has been loaded", dll_base as u64));
                        }
                    } else {
                        print_lg(LevelPrint::Debug, format!("Dll at address : {:#x} has been loaded", dll_base as u64));
                    }
                    
                    match get_export_func_in_dll(h_proc, dll_base as u64) {
                        Ok(export_func) => (*ptr::addr_of_mut!(SYMBOLS_V)).symbol_file.extend_from_slice(&export_func),
                        Err(e) => print_lg(LevelPrint::Error, format!("failed to get export function of dll : {e}")),
                    }
                }
                UNLOAD_DLL_DEBUG_EVENT => {
                    let base_dll = debug_event.u.UnloadDll().lpBaseOfDll;
                    if let Some(pos) = dll_load.iter().position(|d|d.dll_base == base_dll) {
                        let dll = dll_load.remove(pos);
                        print_lg(LevelPrint::Debug, format!("Dll at address : {:#x} has been unloaded ;{}", dll.dll_base as u64, dll.dll_name))
                    }else {
                        print_lg(LevelPrint::Debug, format!("Dll at address : {:#x} has been unloaded", base_dll as u64))
                    }
                    let ret_d = |s: &SymbolFile | {
                        if s.src_file.is_dll() {
                            s.src_file.dll_base() != base_dll as u64
                        }else {
                            true
                        }
                    };
                    (*ptr::addr_of_mut!(SYMBOLS_V)).symbol_file.retain(|s|ret_d(s))
                },
                OUTPUT_DEBUG_STRING_EVENT => {
                    let dbg_strd = debug_event.u.DebugString().lpDebugStringData;
                    let c_str = std::ffi::CStr::from_ptr(dbg_strd as *const c_char);
                    let dbg_str = c_str.to_str().unwrap();
                    print_lg(LevelPrint::Debug, format!("Debug string output : \"{dbg_str}\""));
                }
                CONTROL_C_EXIT => {
                    print_lg(LevelPrint::Debug, "control C exit");
                    stop_dbg(debug_event.dwProcessId);
                }
                _ => {}
            }
            match c_dbg {
                DbgState::Continue => {
                    if ContinueDebugEvent(debug_event.dwProcessId, debug_event.dwThreadId, DBG_CONTINUE) == 0 {
                        print_lg(LevelPrint::Critical1(debug_event.dwProcessId, &mut c_dbg), format!("failed to ContinueDebugEvent : {}", io::Error::last_os_error()));
                        return;
                    }
                }
                DbgState::NeedStop => stop_dbg(debug_event.dwProcessId),
                DbgState::Stopped => return,
            }
        }
    }
}






pub fn start_debugging(cli: &str) {
    unsafe {
        let mut si = mem::zeroed::<STARTUPINFOA>();
        let mut pi = mem::zeroed::<PROCESS_INFORMATION>();
        si.cb = size_of::<STARTUPINFOA>() as u32;
        if CreateProcessA(ptr::null_mut(), cli.as_ptr() as *mut i8, ptr::null_mut(), ptr::null_mut(), 0, DEBUG_PROCESS, ptr::null_mut(), ptr::null_mut(), &mut si, &mut pi) == 0 {
            print_lg(LevelPrint::Error, format!("CreateProcess failed : {}", io::Error::last_os_error()));
            return;
        }
        debug_loop(pi.hProcess);
        CloseHandle(pi.hProcess);
        CloseHandle(pi.hThread);
    }
}
