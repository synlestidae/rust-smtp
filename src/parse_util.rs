use std::io::Read;

use data::{ParseError};

pub fn is_not_space_byte(byte: u8) -> bool {
    byte != 32,
}

pub fn is_space_byte(byte: u8) -> bool {
    byte == 32
}

pub fn is_not_less_than(byte: u8) -> bool {
    byte != ('>' as u8)
}

pub fn ignore_ascii_case(byte_in: u8) -> u8 {
    let mut byte = byte_in;
    if 65 <= byte && byte <= 90 {
        byte += 32;
    } 
    byte
}

pub fn ascii_slice_eq_ignore_case(a_byte: &[u8], b_byte: &[u8]) -> bool {
    a_byte.len() == b_byte.len() && a_byte.iter()
        .zip(b_byte.iter())
        .skip_while(|&(&a, &b)| ascii_eq_ignore_case(a, b))
        .collect::<Vec<_>>()
        .len() == 0
}

pub fn ascii_eq_ignore_case(a_byte: u8, b_byte: u8) -> bool {
    ignore_ascii_case(a_byte) == ignore_ascii_case(b_byte)
}