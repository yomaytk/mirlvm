use once_cell::sync::Lazy;
use std::env;
use std::fs;

type Label = String;
type VarName = String;

const RESERVED_SIZE: usize = 3;
const SIGNALS_SIZE: usize = 7;

pub static RESERVEDWORDS: [(&str, TokenType); RESERVED_SIZE] = [
    ("function", TokenType::Function),
    ("w", TokenType::Word),
    ("ret", TokenType::Ret),
];
pub static SIGNALS: [(&str, TokenType); SIGNALS_SIZE] = [
    ("(", TokenType::Lbrace),
    (")", TokenType::Rbrace),
    ("{", TokenType::Clbrace),
    ("}", TokenType::Crbrace),
    ("$", TokenType::Dollar),
    (":", TokenType::Colon),
    ("@", TokenType::Atm),
];

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TokenType {
    Function,
    Word,
    Ident,
    Instr,
    Lbrace,
    Rbrace,
    Crbrace,
    Clbrace,
    Dollar,
    Ret,
    Ilit,
    Block,
    Colon,
    Atm,
    Eof,
}

#[derive(Debug)]
pub struct TokenMass {
    pub tks: Vec<Token>,
    pub cpos: usize,
}

impl TokenMass {
    pub fn new() -> Self {
        Self { 
            tks: vec![],
            cpos: 0,
        }
    }
    fn push(&mut self, tk: Token) {
        self.tks.push(tk)
    }
    fn assert_tkty(&mut self, tty: TokenType) {
        assert_eq!(self.tks[self.cpos].tty, tty);
        self.cpos += 1;
    }
    fn cur_tkty(&self) -> TokenType {
        self.tks[self.cpos].tty
    }
    fn eq_tkty(&mut self, tk: TokenType) -> bool {
        if self.cur_tkty() == tk {
            self.cpos += 1;
            true
        } else {
            false
        }
    }
    fn getnum_n(&mut self) -> i32 {
        let res = self.tks[self.cpos].num;
        self.cpos += 1;
        res
    }
    fn gettype_n(&mut self) -> VarType {
        let tokentext = self.tks[self.cpos].get_text();
        // There is change for room
        self.cpos += 1;
        match tokentext {
            "w" => { VarType::Word }
            "l" => { VarType::Long }
            _ => { panic!("TokenMass.gettype() error.")}
        }
    }
    fn gettext_n(&mut self) -> &'static str {
        let tktext = self.tks[self.cpos].get_text();
        self.cpos += 1;
        tktext
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Token {
    tty: TokenType,
    poss: usize,
    pose: usize,
    num: i32,
}

impl Token {
    pub fn new(tty: TokenType, poss: usize, pose: usize, num: i32) -> Self {
        Self {
            tty,
            poss,
            pose,
            num,
        }
    }
    fn get_text(&self) -> &'static str {
        &(*PROGRAM)[self.poss..self.pose]
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum VarType {
    Word,
    Long,
    TypeTuple(Vec<VarType>)
}

#[derive(Clone, Debug, PartialEq)]
struct Arg {
    vty: VarType,
    name: VarName,
}

#[derive(Debug)]
pub struct Program {
    pub funcs: Vec<Function>,
}

impl Program {
    pub fn new(funcs: Vec<Function>) -> Self {
        Self {
            funcs,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    pub name: Label,
    pub retty: VarType,
    pub args: Vec<Var>,
    pub bls: Vec<Block>,
}

impl Function {
    pub fn new(name: String, retty: VarType, args: Vec<Var>, bls: Vec<Block>) -> Self {
        Self {
            name,
            retty,
            args,
            bls,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Var {
    pub name: String,
    pub ty: VarType,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    pub lb: String,
    pub instrs: Vec<Instr>
}

impl Block {
    pub fn new(lb: String, instrs: Vec<Instr>) -> Self {
        Self {
            lb,
            instrs
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Instr {
    Ret(i32)
}

pub static PROGRAM: Lazy<String> = Lazy::new(|| {
    let file: String = env::args().collect::<Vec<String>>().last().unwrap().clone();
    fs::read_to_string(file).expect("failed to read file.")
});

fn lex() -> TokenMass {
    let program = (*PROGRAM).clone();
    let pgchars: Vec<char> = (&program[..]).chars().collect();
    let pglen = program.len();
    let mut pos = 0;
    let mut tmass = TokenMass::new();

    loop {
        if pos == pglen {
            break;
        }
        if pgchars[pos].is_whitespace() {
            pos += 1;
            continue;
        }
        // identification or reserved words
        if pgchars[pos].is_ascii_alphabetic() {
            let mut pose = pos;
            let mut tty = TokenType::Ident;
            while pgchars[pose].is_ascii_alphanumeric() {
                pose += 1;
            }
            for i in 0..RESERVED_SIZE {
                if RESERVEDWORDS[i].0 == &program[pos..pose] {
                    tty = RESERVEDWORDS[i].1;
                    break;
                }
            }
            tmass.push(Token::new(tty, pos, pose, -1));
            pos = pose;
            continue;
        }
        // integer
        if pgchars[pos].is_ascii_digit() {
            let mut num = 0;
            let poss = pos;
            while pgchars[pos].is_ascii_digit() {
                num = num * 10 + (pgchars[pos] as i32 - 48);
                pos += 1;
            }
            tmass.push(Token::new(TokenType::Ilit, poss, pos, num));
            continue;
        }
        // signals
        let mut nextloop = false;
        for i in 0..SIGNALS_SIZE {
            let pose = pos + 1;
            if SIGNALS[i].0 == &program[pos..pose] {
                let tty = SIGNALS[i].1;
                nextloop = true;
                tmass.push(Token::new(tty, pos, pose, -1));
                pos = pose;
                break;
            }
        }
        if nextloop {
            continue;
        }

        panic!("failed lex program.");
    }
    tmass.push(Token::new(TokenType::Eof, 0, 0, -1));
    tmass
}

fn parseargs(tmass: &mut TokenMass) -> Vec<Var> {
    let argvars = vec![];
    tmass.assert_tkty(TokenType::Lbrace);
    // parse each arguments
    loop {
        break;
    }
    tmass.assert_tkty(TokenType::Rbrace);
    argvars
}

fn parseinstr(tmass: &mut TokenMass) -> Instr {
    if tmass.eq_tkty(TokenType::Ret) {
        let retnum = tmass.getnum_n();
        Instr::Ret(retnum)
    } else {
        panic!("parseinstr panic");
    }
}

// parse basic block
fn parsebb(tmass: &mut TokenMass) -> Block {
    tmass.assert_tkty(TokenType::Atm);
    let blocklb = tmass.gettext_n();
    tmass.assert_tkty(TokenType::Colon);
    let mut instrs = vec![];
    loop {
        let tkty = tmass.cur_tkty();
        if tkty == TokenType::Atm
        || tkty == TokenType::Crbrace {
            break;
        }
        // panic!("{:?}", tmass.tks[tmass.cpos]);
        instrs.push(parseinstr(tmass));
    }
    Block::new(String::from(blocklb), instrs)
}

// parse function ...
fn parsefun(tmass: &mut TokenMass) -> Function {
    tmass.assert_tkty(TokenType::Function);
    let functy = tmass.gettype_n();
    tmass.assert_tkty(TokenType::Dollar);
    let funclb = tmass.gettext_n();
    // parse arguments
    let argvars = parseargs(tmass);
    // function body
    tmass.assert_tkty(TokenType::Clbrace);
    let mut blocks = vec![];
    loop {
        let ctkty = tmass.cur_tkty();
        if ctkty == TokenType::Atm {
                let bblock = parsebb(tmass);
                blocks.push(bblock);
        } else {
            tmass.assert_tkty(TokenType::Crbrace);
            break;
        }
    }
    Function::new(String::from(funclb), functy, argvars, blocks)
}

fn parse(tmass: &mut TokenMass) -> Program {
    let mut funcs = vec![];
    loop {
        if tmass.cur_tkty() == TokenType::Function {
            funcs.push(parsefun(tmass));
            continue;
        }
        tmass.assert_tkty(TokenType::Eof);
        break;
    }
    Program::new(funcs)
}

fn main() {
    let mut tmass = lex();
    // println!("{:#?}", tmass);
    let program = parse(&mut tmass);
    println!("{:?}", program);
}