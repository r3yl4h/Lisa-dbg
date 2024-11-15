use std::fmt;
use std::num::ParseIntError;
use std::str::FromStr;
use anyhow::{anyhow, Error};
use num::Num;
use crate::ut::fmt::fmt;



pub trait ToType {
    fn from_str_value(value: &str) -> Result<Self, Error>
    where
        Self: Sized;
    fn from_char(value: char) -> Self
    where
        Self: Sized;
}

impl ToType for u8 {
    fn from_str_value(value: &str) -> Result<Self, Error> {
        str_to(value)
    }
    fn from_char(value: char) -> Self {
        str_to(&(value as u8).to_string()).unwrap()
    }
}

impl ToType for i8 {
    fn from_str_value(value: &str) -> Result<Self, Error> {
        str_to(value)
    }

    fn from_char(value: char) -> Self {
        str_to(&(value as u8).to_string()).unwrap()
    }
}

impl ToType for u16 {
    fn from_str_value(value: &str) -> Result<Self, Error> {
        str_to(value)
    }

    fn from_char(value: char) -> Self {
        str_to(&(value as u8).to_string()).unwrap()
    }
}

impl ToType for i16 {
    fn from_str_value(value: &str) -> Result<Self, Error> {
        str_to(value)
    }

    fn from_char(value: char) -> Self {
        str_to(&(value as u8).to_string()).unwrap()
    }
}

impl ToType for u32 {
    fn from_str_value(value: &str) -> Result<Self, Error> {
        str_to(value)
    }

    fn from_char(value: char) -> Self {
        str_to(&(value as u8).to_string()).unwrap()
    }
}

impl ToType for i32 {
    fn from_str_value(value: &str) -> Result<Self, Error> {
        str_to(value)
    }

    fn from_char(value: char) -> Self {
        str_to(&(value as u8).to_string()).unwrap()
    }
}

impl ToType for u64 {
    fn from_str_value(value: &str) -> Result<Self, Error> {
        str_to(value)
    }

    fn from_char(value: char) -> Self {
        str_to(&(value as u8).to_string()).unwrap()
    }
}

impl ToType for i64 {
    fn from_str_value(value: &str) -> Result<Self, Error> {
        str_to(value)
    }

    fn from_char(value: char) -> Self {
        str_to(&(value as u8).to_string()).unwrap()
    }
}

impl ToType for f32 {
    fn from_str_value(value: &str) -> Result<Self, Error> {
        Ok(f32::from_str(value)?)
    }

    fn from_char(value: char) -> Self {
        f32::from_str(&(value as u8).to_string()).unwrap()
    }
}

impl ToType for f64 {
    fn from_str_value(value: &str) -> Result<Self, Error> {
        Ok(f64::from_str(value)?)
    }

    fn from_char(value: char) -> Self {
        f64::from_str(&(value as u8).to_string()).unwrap()
    }
}

impl ToType for char {
    fn from_str_value(value: &str) -> Result<Self, Error> {
        let c = if value != "" {
            value.as_bytes()[0] as char
        } else {
            '\0'
        };
        Ok(c)
    }

    fn from_char(value: char) -> Self {
        value
    }
}



impl ToType for bool {
    fn from_str_value(value: &str) -> Result<Self, Error> {
        Ok(value == "true")
    }

    fn from_char(value: char) -> Self {
        value == '1'
    }
}



pub trait NumConvert {
    fn to_u64(&self) -> u64;
    fn from_u64(value: u64) -> Self;
}

impl NumConvert for u32 {
    fn to_u64(&self) -> u64 {
        *self as u64
    }

    fn from_u64(value: u64) -> Self {
        value as u32
    }
}

impl NumConvert for u64 {
    fn to_u64(&self) -> u64 {
        *self
    }

    fn from_u64(value: u64) -> Self {
        value
    }
}



#[derive(Default, Clone, PartialEq)]
pub struct Char(pub u8);

impl fmt::Display for Char {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt(self, f)
    }
}

impl fmt::Debug for Char {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt(self, f)
    }
}


impl ToType for Char {
    fn from_str_value(value: &str) -> Result<Self, Error> {
        Ok(Char(str_to(value)?))
    }

    fn from_char(value: char) -> Self {
        Char(value as u8)
    }
}

pub fn str_to<T: Num<FromStrRadixErr = ParseIntError> + FromStr<Err = ParseIntError>>(addr_str: &str) -> Result<T, Error> {
    if addr_str.to_lowercase().starts_with("0x") {
        match T::from_str_radix(&addr_str.replace("0x", ""), 16) {
            Ok(value) => Ok(value),
            Err(e) => Err(anyhow!("{e} : {addr_str}")),
        }
    } else if addr_str.to_lowercase().ends_with("h") {
        match T::from_str_radix(&addr_str[..addr_str.len() - 1], 16) {
            Ok(value) => Ok(value),
            Err(e) => Err(anyhow!("{e} : {addr_str}")),
        }
    } else {
        Ok(addr_str.parse()?)
    }
}