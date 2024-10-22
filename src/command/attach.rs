use crate::utils::{str_to, ERR_COLOR, RESET_COLOR};
use crate::{dbg, process, usage, ALL_ELM};

pub fn handle_attach(linev: &[&str]) {
    if linev.len() < 2 {
        println!("{}", usage::USAGE_ATTACH);
        return;
    }
    unsafe { ALL_ELM.attach = Some("".to_string()) }

    let pid = match str_to::<u32>(&linev[1]) {
        Ok(pid) => pid,
        Err(_) => {
            if linev[1].contains("\"") {
                match process::get_pid_with_name(&linev[1].replace("\"", "")) {
                    Ok(pid) => pid,
                    Err(e) => {
                        eprintln!("{ERR_COLOR}{e}{RESET_COLOR}");
                        return;
                    }
                }
            } else {
                eprintln!("{ERR_COLOR}Invalid target{RESET_COLOR}");
                return;
            }
        }
    };
    unsafe { dbg::attach::attach_dbg(pid) }
}
