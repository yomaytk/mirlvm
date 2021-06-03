use once_cell::sync::Lazy;
use std::env;
use std::fs;

pub mod codegen;
pub mod lexer;
pub mod lowir;
pub mod parser;
pub mod rega;

type Label = &'static str;
type VarName = String;

pub static PROGRAM: Lazy<String> = Lazy::new(|| {
    let file: String = env::args().collect::<Vec<String>>().last().unwrap().clone();
    fs::read_to_string(file).expect("failed to read file.")
});
