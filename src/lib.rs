use std::fs;
use std::env;
use once_cell::sync::Lazy;

pub mod lexer;
pub mod parser;
pub mod lowir;
pub mod rega;

type Label = String;
type VarName = String;
type ByteSize = i32;

pub static PROGRAM: Lazy<String> = Lazy::new(|| {
    let file: String = env::args().collect::<Vec<String>>().last().unwrap().clone();
    fs::read_to_string(file).expect("failed to read file.")
});