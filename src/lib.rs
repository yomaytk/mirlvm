use once_cell::sync::Lazy;
use std::env;
use std::fs;

pub mod codegen;
pub mod deadcode;
pub mod dominators;
pub mod lexer;
pub mod lowir;
pub mod mem2reg;
pub mod parser;
pub mod rega;
pub mod rev_ssa;

type Label = &'static str;
type VarName = &'static str;
type BBLabel = &'static str;

pub static PROGRAM: Lazy<String> = Lazy::new(|| {
    let file: String = env::args().collect::<Vec<String>>().last().unwrap().clone();
    fs::read_to_string(file).expect("failed to read file.")
});
