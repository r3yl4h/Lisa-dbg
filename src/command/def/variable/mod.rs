pub mod variable;
pub mod printf;

use crate::command::def::types::TypeP;

#[derive(Debug, Default, Clone)]
pub struct Var {
    type_p: TypeP,
    name: String,
    value: Vec<u8>
}


#[allow(dead_code)]
impl Var {
    pub fn to_u8(&self) -> Result<u8, anyhow::Error> {
        Ok(u8::from_le_bytes(self.value[0..1].try_into()?))
    }
    
    pub fn to_i8(&self) -> Result<i8, anyhow::Error> {
        Ok(i8::from_le_bytes(self.value[0..1].try_into()?))
    }

    pub fn to_u16(&self) -> Result<u16, anyhow::Error>  {
        Ok(u16::from_le_bytes(self.value[0..2].try_into()?))
    }

    pub fn to_i16(&self) -> Result<i16, anyhow::Error> {
        Ok(i16::from_le_bytes(self.value[0..2].try_into()?))
    }
    
    
    pub fn to_u32(&self) -> Result<u32, anyhow::Error> {
        Ok(u32::from_le_bytes(self.value[0..4].try_into()?))
    }
    
    pub fn to_i32(&self) -> Result<i32, anyhow::Error> {
        Ok(i32::from_le_bytes(self.value[0..4].try_into()?))
    }

    pub fn to_u64(&self) -> Result<u64, anyhow::Error> {
        Ok(u64::from_le_bytes(self.value[0..8].try_into()?))
    }
    
    pub fn to_i64(&self) -> Result<i64, anyhow::Error> {
        Ok(i64::from_le_bytes(self.value[0..8].try_into()?))
    }
    
    pub fn to_f32(&self) -> Result<f32, anyhow::Error> {
        Ok(f32::from_le_bytes(self.value[0..4].try_into()?))
    }
    
    pub fn to_f64(&self) -> Result<f64, anyhow::Error> {
        Ok(f64::from_le_bytes(self.value[0..4].try_into()?))
    }
    
    pub fn get_var_with_name<T: IntoIterator<Item = Var>>(vec: T, name: &str) -> Option<Var> {
        vec.into_iter().find(|v|v.name == name)
    }
}

