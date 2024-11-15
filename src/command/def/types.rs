use std::fmt;
use std::str::FromStr;
use regex::Regex;
use crate::pefile::{NtHeaders, NT_HEADER};
use crate::ut::cast::str_to;
use crate::ut::fmt::{print_lg, LevelPrint};

#[derive(Debug, Clone)]
pub enum TypeP {
    U8(usize),
    U16(usize),
    U32(usize),
    U64(usize),
    I8(usize),
    I16(usize),
    I32(usize),
    I64(usize),
    F32(usize),
    F64(usize),
    Char(usize),
    Structs(Vec<StructP>, String),
    Bool(usize),
    Ptr(Box<PtrS>, usize),
    Void,
}

impl Default for TypeP {
    fn default() -> Self {
        Self::Void
    }
}

#[derive(Debug, Default, Clone)]
pub struct PtrS {
    pub cout_ptr: usize,
    pub type_deref: Box<TypeP>,
}

#[derive(Debug, Default, Clone)]
pub struct StructP {
    pub name_field: String,
    pub type_p: TypeP,
}

impl fmt::Display for TypeP {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeP::U8(size) => write_type(f, "uint8_t", *size),
            TypeP::U16(size) => write_type(f, "uint16_t", *size),
            TypeP::U32(size) => write_type(f, "uint32_t", *size),
            TypeP::U64(size) => write_type(f, "uint64_t", *size),
            TypeP::I8(size) => write_type(f, "int8_t", *size),
            TypeP::I16(size) => write_type(f, "int16_t", *size),
            TypeP::I32(size) => write_type(f, "int32_t", *size),
            TypeP::I64(size) => write_type(f, "int64_t", *size),
            TypeP::F32(size) => write_type(f, "float", *size),
            TypeP::F64(size) => write_type(f, "double", *size),
            TypeP::Char(size) => write_type(f, "char", *size),
            TypeP::Bool(size) => write_type(f, "bool", *size),
            TypeP::Ptr(ptr, size) => {
                let mut result = format!("{}", *ptr.type_deref);
                for _ in 0..ptr.cout_ptr {
                    result.push('*');
                }
                if *size != 1 {
                    write!(f, "{}[{}]", result, size)
                } else {
                    write!(f, "{}", result)
                }
            }
            TypeP::Structs(_, name) => write!(f, "struct {}", name),
            TypeP::Void => write!(f, "void"),
        }
    }
}

fn write_type(f: &mut fmt::Formatter<'_>, type_name: &str, size: usize) -> fmt::Result {
    if size != 1 {
        write!(f, "{}[{}]", type_name, size)
    } else {
        write!(f, "{}", type_name)
    }
}

impl TypeP {
    pub fn get_size(&self) -> usize {
        match self {
            TypeP::U8(cout) | TypeP::I8(cout) | TypeP::Bool(cout) | TypeP::Char(cout) => cout * 1,
            TypeP::U16(cout) | TypeP::I16(cout) => cout * 2,
            TypeP::U32(cout) | TypeP::I32(cout) | TypeP::F32(cout) => cout * 4,
            TypeP::U64(cout) | TypeP::I64(cout) | TypeP::F64(cout) => cout * 8,
            TypeP::Ptr(_, cout) => {
                if let Some(nt) = unsafe { NT_HEADER } {
                    match nt {
                        NtHeaders::Headers32(_) => 4 * cout,
                        NtHeaders::Headers64(_) => 8 * cout,
                    }
                } else {
                    print_lg(LevelPrint::WarningO, "you have not defined a file, we cannot know if the target architecture is 32b or 64b");
                    0
                }
            }
            TypeP::Structs(vtypes, _) => {
                let mut result = 0;
                for types in vtypes {
                    result += types.type_p.get_size();
                }
                result
            }
            TypeP::Void => 0,
        }
    }

    pub fn set_cout(&mut self, new_cout: usize) {
        match self {
            TypeP::U8(_) => *self = TypeP::U8(new_cout),
            TypeP::U16(_) => *self = TypeP::U16(new_cout),
            TypeP::U32(_) => *self = TypeP::U32(new_cout),
            TypeP::U64(_) => *self = TypeP::U64(new_cout),
            TypeP::I8(_) => *self = TypeP::I8(new_cout),
            TypeP::I16(_) => *self = TypeP::I16(new_cout),
            TypeP::I32(_) => *self = TypeP::I32(new_cout),
            TypeP::I64(_) => *self = TypeP::I64(new_cout),
            TypeP::F32(_) => *self = TypeP::F32(new_cout),
            TypeP::F64(_) => *self = TypeP::F64(new_cout),
            TypeP::Char(_) => *self = TypeP::Char(new_cout),
            TypeP::Bool(_) => *self = TypeP::Bool(new_cout),
            _ => {}
        }
    }
    
    #[allow(dead_code)]
    pub fn is_ptr_castable(&self) -> bool {
        match self {
            TypeP::Ptr(_, _) | TypeP::U64(_) | TypeP::I64(_) => true,
            _ => false,
        }
    }
    
    #[allow(dead_code)]
    pub fn is_ptr_absolute(&self) -> bool {
        match self { 
            TypeP::Ptr(_,_) => true,
            _ => false,
        }
    }
    
    pub fn size_of_type(&self) -> usize {
        let size = self.get_size();
        let cout = self.cout_elm();
        size / cout
    }
    
    pub fn cout_elm(&self) -> usize {
        match self {
            TypeP::U8(cout) | TypeP::I8(cout) | TypeP::Bool(cout) 
            | TypeP::Char(cout) | TypeP::U16(cout) | TypeP::I16(cout)
            | TypeP::U32(cout) | TypeP::I32(cout) | TypeP::F32(cout)
            | TypeP::U64(cout) | TypeP::I64(cout) | TypeP::F64(cout) | TypeP::Ptr(_, cout) => *cout,
            TypeP::Void | TypeP::Structs(_, _)  => 0,
        }
    }

    pub fn get_name_of_struct(&self) -> String {
        match self {
            TypeP::Structs(_, name) => name.clone(),
            _ => "".to_string(),
        }
    }

    pub fn get_field_of_struct(&self) -> Vec<StructP> {
        match self {
            TypeP::Structs(field, _) => field.clone(),
            _ => Vec::new(),
        }
    }

    pub fn get_type_with_str(type_str: &str, cout: usize) -> Result<TypeP, String> {
        match type_str {
            "u8" | "uint8_t" | "byte" => Ok(TypeP::U8(cout)),
            "i8" | "int8_t" => Ok(TypeP::I8(cout)),
            "char" => Ok(TypeP::Char(cout)),
            "u16" | "uint16_t" | "word" => Ok(TypeP::U16(cout)),
            "i16" | "int16_t" | "short" => Ok(TypeP::I16(cout)),
            "u32" | "uint32_t" | "dword" => Ok(TypeP::U32(cout)),
            "i32" | "int32_t" | "int" => Ok(TypeP::I32(cout)),
            "u64" | "uint64_t" | "qword" => Ok(TypeP::U64(cout)),
            "i64" | "int64_t" | "long long" => Ok(TypeP::I64(cout)),
            "f32" | "float" => Ok(TypeP::F32(cout)),
            "f64" | "double" => Ok(TypeP::F64(cout)),
            "bool" => Ok(TypeP::Bool(cout)),
            "void" => Ok(TypeP::Void),
            _ => Err(format!("Unknown type p: {}", type_str)),
        }
    }
}

impl FromStr for TypeP {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let vs: Vec<&str> = s.split_whitespace().collect();
        if vs.len() == 2 {
            let types = vs[0].to_lowercase();
            let mut cout = 1;
            let re = Regex::new(r"\[(.*?)]").unwrap();
            for cap in re.captures_iter(&vs[1]) {
                if let Some(num) = cap.get(1) {
                    match str_to::<usize>(num.as_str()) {
                        Ok(num) => cout = num,
                        Err(e) => return Err(e.to_string()),
                    }
                } else {
                    continue;
                }
            }
            if types.contains("*") {
                let cout_ptr = types.matches("*").count();
                let types_d = types.replace("*", "");
                let ptr_type = TypeP::get_type_with_str(&types_d, cout)?;
                return Ok(TypeP::Ptr(Box::new(PtrS { cout_ptr, type_deref: Box::new(ptr_type) }), cout));
            }
            Ok(TypeP::get_type_with_str(&types, cout)?)
        } else {
            Err("valid line is <type> <field name>[cout]".to_string())
        }
    }
}