use crate::{dbg, process, usage, ALL_ELM};
use crate::ut::cast::str_to;
use crate::ut::fmt::{print_lg, LevelPrint};

pub fn handle_attach(linev: &[&str]) {
    if linev.len() < 2 {
        println!("{}", usage::USAGE_ATTACH);
        return;
    }
    unsafe { ALL_ELM.attach = Some("".to_string()); }

    let pid = match str_to::<u32>(&linev[1]) {
        Ok(pid) => pid,
        Err(_) => {
            if linev[1].contains("\"") {
                match process::get_pid_with_name(&linev[1].replace("\"", "")) {
                    Ok(pid) => pid,
                    Err(e) => {
                        print_lg(LevelPrint::ErrorO, e);
                        return;
                    }
                }
            } else {
                print_lg(LevelPrint::ErrorO, "Invalid target");
                return;
            }
        }
    };
    unsafe { dbg::attach::attach_dbg(pid) }
}
