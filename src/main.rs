use once_cell::sync::Lazy;
use std::env;
use std::fs;
use std::collections::HashMap;

type Label = String;
type VarName = String;

const RESERVED_SIZE: usize = 7;
const SIGNALS_SIZE: usize = 10;

pub static RESERVEDWORDS: [(&str, TokenType); RESERVED_SIZE] = [
    ("function", TokenType::Function),
    ("w", TokenType::Word),
    ("ret", TokenType::Ret),
    ("alloc4", TokenType::Alloc4),
    ("storew", TokenType::Storew),
    ("loadw", TokenType::Loadw),
    ("add", TokenType::Add),
];

pub static SIGNALS: [(&str, TokenType); SIGNALS_SIZE] = [
    ("(", TokenType::Lbrace),
    (")", TokenType::Rbrace),
    ("{", TokenType::Clbrace),
    ("}", TokenType::Crbrace),
    ("$", TokenType::Dollar),
    (":", TokenType::Colon),
    ("@", TokenType::Atm),
    ("=w", TokenType::Eqw),
    ("=l", TokenType::Eql),
    (",", TokenType::Comma),
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
    Alloc4,
    Eql,
    Eqw,
    Add,
    Storew,
    Loadw,
    Comma,
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
        assert_eq!(self.tks[self.cpos].tty, TokenType::Ilit);
        let res = self.tks[self.cpos].num;
        self.cpos += 1;
        res
    }
    fn getvar_n(&mut self, varenv: &VariableEnvironment) -> Var {
        let key = self.tks[self.cpos].get_text();
        let res = varenv.get(key);
        self.cpos += 1;
        res
    }
    fn getfirstclassobj_n(&mut self, varenv: &VariableEnvironment) -> FirstClassObj {
        let tty = self.tks[self.cpos].tty;
        self.cpos += 1;
        if tty == TokenType::Ident {
            return FirstClassObj::Variable(varenv.get(self.tks[self.cpos-1].get_text()));
        }
        if tty == TokenType::Ilit {
            return FirstClassObj::Num(self.tks[self.cpos-1].num);
        }
        panic!("getfirstclassobj_n error. {:?}", self.tks[self.cpos-1]);
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
    fn getcurrent_token(&self) -> Token {
        self.tks[self.cpos]
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
    pub name: &'static str,
    pub ty: VarType,
}

impl Var {
    pub fn new(name: &'static str, ty: VarType) -> Self {
        Self {
            name,
            ty,
        }
    }
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
pub enum FirstClassObj {
    Variable(Var),
    Num(i32),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Instr {
    Ret(FirstClassObj),
    Assign(AssignType, Var, Box<Instr>),
    Alloc4(i32),
    Storew(FirstClassObj, Var),
    Loadw(Var),
    Add(FirstClassObj, FirstClassObj),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AssignType {
    Word,
    Long,
}

#[derive(Clone, Debug, PartialEq)]
pub struct VariableEnvironment {
    vars: HashMap<&'static str, Var>,
}

impl VariableEnvironment {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new()
        }
    }
    fn get(&self, key: &'static str) -> Var {
        // self.vars.get(key).unwrap().clone()
        let r = self.vars.get(key);
        if let Some(v) = r {
            return v.clone();
        } else {
            panic!("{:?}", key);
        }
    }
    fn append(&mut self, var: Var) {
        self.vars.insert(var.name, var);
    }
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
        if pgchars[pos] == '%' || pgchars[pos].is_ascii_alphabetic()     {
            let mut pose = pos;
            let mut tty = TokenType::Ident;
            pose += 1;
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
            let signal = SIGNALS[i].0;
            let pose = pos + signal.len();
            if signal == &program[pos..pose] {
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

        panic!("failed lex program. next letter = {}", pgchars[pos]);
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

// parser rhs of instr
fn parseinstrrhs(tmass: &mut TokenMass, varenv: &mut VariableEnvironment) -> Instr {
    // alloc4
    if tmass.eq_tkty(TokenType::Alloc4) {
        let rhs = tmass.getnum_n();
        return Instr::Alloc4(rhs);
    }
    // loadw
    if tmass.eq_tkty(TokenType::Loadw) {
        let rhs = tmass.getvar_n(varenv);
        return Instr::Loadw(rhs);
    }
    // add
    if tmass.eq_tkty(TokenType::Add) {
        let lhs = tmass.getfirstclassobj_n(varenv);
        tmass.assert_tkty(TokenType::Comma);
        let rhs = tmass.getfirstclassobj_n(varenv);
        return Instr::Add(lhs, rhs);
    }
    let curtk = tmass.getcurrent_token();
    panic!("parseinstr panic {:?}: {}", curtk, &PROGRAM[curtk.poss..curtk.pose]);
}

fn parseinstroverall(tmass: &mut TokenMass, varenv: &mut VariableEnvironment) -> Instr {
    // ret
    if tmass.eq_tkty(TokenType::Ret) {
        let retnum = tmass.getfirstclassobj_n(varenv);
        return Instr::Ret(retnum)
    }
    // lhs =* rhs instruction
    if tmass.cur_tkty() == TokenType::Ident {
        let varname = tmass.gettext_n();
        let cur_tkty = tmass.cur_tkty();
        let assignty;
        let var;
        if cur_tkty == TokenType::Eql { 
            assignty = AssignType::Long;
            var = Var::new(varname, VarType::Long);
        }
        else {
            assert_eq!(cur_tkty, TokenType::Eqw);
            assignty = AssignType::Word;
            var = Var::new(varname, VarType::Word);
        }
        tmass.cpos += 1;
        let rhs = parseinstrrhs(tmass, varenv);
        varenv.append(var.clone());
        return Instr::Assign(assignty, var, Box::new(rhs));
    }
    // storew
    if tmass.eq_tkty(TokenType::Storew) {
        let lhs = tmass.getfirstclassobj_n(varenv);
        tmass.assert_tkty(TokenType::Comma);
        let rhs = tmass.getvar_n(varenv);
        return Instr::Storew(lhs, rhs);
    }
    panic!("parseinstroverall error. {:?}", tmass.getcurrent_token());
}

// parse basic block
fn parsebb(tmass: &mut TokenMass) -> Block {
    tmass.assert_tkty(TokenType::Atm);
    let blocklb = tmass.gettext_n();
    tmass.assert_tkty(TokenType::Colon);
    let mut instrs = vec![];
    let mut varenv = VariableEnvironment::new();
    loop {
        let tkty = tmass.cur_tkty();
        if tkty == TokenType::Atm
        || tkty == TokenType::Crbrace {
            break;
        }
        instrs.push(parseinstroverall(tmass, &mut varenv));
        // panic!("fjewiojfwejfwjefj");
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
    println!("{:#?}", program);
}