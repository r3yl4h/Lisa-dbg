use crate::usage::USAGE_DEF_STRUCT;
use crate::ALL_ELM;
use std::io::Write;
use std::str::FromStr;
use std::{io, ptr};
use crate::command::def::types::{StructP, TypeP};
use crate::ut::fmt::{print_lg, LevelPrint};





pub fn def_struct(linev: &[&str]) {
    if linev.len() < 2 {
        println!("{USAGE_DEF_STRUCT}");
        return;
    }
    let mut field_type = Vec::new();
    let mut field_name = Vec::new();
    println!("struct {} {{", linev[1]);
    loop {
        let mut input = String::new();
        print!("    ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        if input == ":q" {
            print!("\x1B[1A\x1B[2K");
            break;
        }
        match TypeP::from_str(input) {
            Ok(t) => {
                let inp = input.replace(";", "");
                let name = inp.split_whitespace().collect::<Vec<&str>>()[1].split('[').next().unwrap();
                field_name.push(name.to_string());
                field_type.push(t);
                if !input.ends_with(";") {
                    print!("\x1B[1A\x1B[2K");
                    println!("    {input};");
                }
            }
            Err(e) => {
                print!("\x1B[1A\x1B[2K");
                print_lg(LevelPrint::ErrorO, format!("    {input} # {e}"));
            }
        }
    }
    println!("}}");
    let mut struct_s = Vec::new();
    for i in 0..field_type.len() {
        struct_s.push(StructP {
            name_field: field_name[i].to_string(),
            type_p: field_type[i].clone(),
        })
    }

    unsafe { (*ptr::addr_of_mut!(ALL_ELM)).struct_def.push(TypeP::Structs(struct_s, linev[1].to_string())); }
}
