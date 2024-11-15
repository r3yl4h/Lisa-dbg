use std::fs::File;
use std::io::Read;
use std::os::windows::fs::MetadataExt;
use std::str::FromStr;
use crate::cli::ALL_ELM;
use crate::command::def::types::TypeP;
use crate::command::def::variable::Var;
use crate::usage::USAGE_VAR_DEF;
use crate::ut::cast::str_to;
use crate::ut::fmt::*;

pub fn handle_var(linev: &[&str]) {
    if linev.len() < 2 {
        print_lg(LevelPrint::DebugO, USAGE_VAR_DEF);
        return;
    }
    let mut var_st = Var::default();
    let last_pr = linev.join(" ");
    let b = last_pr.replace("=", " ");
    let mut elm: Vec<&str> = b.split_whitespace().collect();
    if elm.len() < 3 {
        print_lg(LevelPrint::ErrorO, "you must declare the type, name and value of the variable like this: type myvar = value");
        return;
    }
    let last = elm.last_mut().unwrap();
    let mut last_st = last.to_string();
    if last.ends_with(";") {
        last_st.pop();
        *last = &last_st;
    }
    let mut guess_cout = false;
    if let Some(type_t) = unsafe {(*&raw const ALL_ELM).struct_def.iter().find(|t|t.get_name_of_struct() == elm[1])} {
        var_st.type_p = type_t.clone();
        var_st.name = type_t.get_name_of_struct();
    }else {
        match TypeP::from_str(&elm[0..2].join(" ")) {
            Ok(types) => {
                var_st.name = elm[1].split("[").next().unwrap().to_string();
                var_st.type_p = types;
            },
            Err(e) => {
                if elm[1].ends_with("[]") {
                    guess_cout = true;
                    match TypeP::get_type_with_str(&elm[0], 0) {
                        Ok(types) => var_st.type_p = types,
                        Err(e) => {
                            print_lg(LevelPrint::ErrorO, &format!("Could not parse type: {}", e));
                            return;
                        }
                    }
                    var_st.name = elm[1].split("[").next().unwrap().to_string();
                }else {
                    print_lg(LevelPrint::ErrorO, e);
                    return;
                }
            },
        }
    }
    if elm.len() > 3 {
        match elm[3] {
            "read" => {
                match File::open(linev[3..].join(" ")) {
                    Ok(mut file) => {
                        let mut buf = if elm.len() > 4 {
                            match str_to::<usize>(elm[4]) {
                                Ok(size) => vec![0u8;size],
                                Err(e) => {
                                    print_lg(LevelPrint::ErrorO, e);
                                    return;
                                }
                            }
                        }else {
                            vec![0u8; file.metadata().unwrap().file_size() as usize]
                        };
                        if let Err(e) = file.read_exact(&mut buf) {
                            print_lg(LevelPrint::ErrorO, format!("failed to read file : {e}"));
                            return;
                        }
                        var_st.value = buf;
                    },
                    Err(e) => print_lg(LevelPrint::ErrorO, e),
                }
            }
            _ => {}
        }
    } else {
        match get_value2vec(elm[2], &mut var_st.type_p, guess_cout) {
            Ok(value) => {
                var_st.value = value;
                unsafe {(*&raw mut ALL_ELM).var_def.push(var_st)};
            },
            Err(e) => print_lg(LevelPrint::ErrorO, e),
        }
    }
}


pub fn get_value2vec(value_str: &str, type_p: &mut TypeP, guess_cout: bool) -> Result<Vec<u8>, anyhow::Error> {
    let mut result = Vec::new();
    if value_str.starts_with("\"") {
        if type_p.is_ptr_castable() {
            if let Some(mut pos) = value_str[1..].find('"') {
                pos += 1;
                let value = &value_str[..pos];
                *type_p = TypeP::Char(value.len());
                result.copy_from_slice(value.as_bytes());
            }
        }
    }else if value_str.starts_with("{") {
        if let Some(mut pos) = value_str[1..].find('}') {
            pos += 1;
            let value = &value_str[..pos];
            let vec_v = value.split(",").collect::<Vec<&str>>();
            for elm in &vec_v {
                if elm.starts_with("'") {
                    result.push(elm[1..2].chars().collect::<Vec<char>>()[0] as u8);
                } else {
                    match str_to::<i128>(elm) {
                        Ok(value) => result.extend_from_slice(&value.to_le_bytes()[0..type_p.size_of_type()]),
                        Err(e) => print_lg(LevelPrint::ErrorO, e),
                    }
                }
            }
            if guess_cout {
                type_p.set_cout(vec_v.len());
            }
        } else {
            return Err(anyhow::Error::msg(format!("{{ not closed {}", &value_str[1..])))
        }
    } else {
        match str_to::<i128>(value_str) {
            Ok(value) => result.extend_from_slice(&value.to_le_bytes()[0..type_p.size_of_type()]),
            Err(e) => print_lg(LevelPrint::ErrorO, e),
        }
    }
    Ok(result)
}