use super::*;
use super::parser::{VariableEnvironment, FirstClassObj, VarType, Var};

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
    pub fn assert_tkty(&mut self, tty: TokenType) {
        assert_eq!(self.tks[self.cpos].tty, tty);
        self.cpos += 1;
    }
    pub fn cur_tkty(&self) -> TokenType {
        self.tks[self.cpos].tty
    }
    pub fn eq_tkty(&mut self, tk: TokenType) -> bool {
        if self.cur_tkty() == tk {
            self.cpos += 1;
            true
        } else {
            false
        }
    }
    pub fn getnum_n(&mut self) -> i32 {
        assert_eq!(self.tks[self.cpos].tty, TokenType::Ilit);
        let res = self.tks[self.cpos].num;
        self.cpos += 1;
        res
    }
    pub fn getvar_n(&mut self, varenv: &VariableEnvironment<&'static str, Var>) -> Var {
        let key = self.tks[self.cpos].get_text();
        let res = varenv.get(&key);
        self.cpos += 1;
        res
    }
    pub fn getfirstclassobj_n(&mut self, varenv: &VariableEnvironment<&'static str, Var>) -> FirstClassObj {
        let tty = self.tks[self.cpos].tty;
        self.cpos += 1;
        if tty == TokenType::Ident {
            return FirstClassObj::Variable(varenv.get(&self.tks[self.cpos-1].get_text()));
        }
        if tty == TokenType::Ilit {
            return FirstClassObj::Num(self.tks[self.cpos-1].num);
        }
        panic!("getfirstclassobj_n error. {:?}", self.tks[self.cpos-1]);
    }
    pub fn gettype_n(&mut self) -> VarType {
        let tokentext = self.tks[self.cpos].get_text();
        // There is change for room
        self.cpos += 1;
        match tokentext {
            "w" => { VarType::Word }
            "l" => { VarType::Long }
            _ => { panic!("TokenMass.gettype() error.")}
        }
    }
    pub fn gettext_n(&mut self) -> &'static str {
        let tktext = self.tks[self.cpos].get_text();
        self.cpos += 1;
        tktext
    }
    pub fn getcurrent_token(&self) -> Token {
        self.tks[self.cpos]
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Token {
    pub tty: TokenType,
    pub poss: usize,
    pub pose: usize,
    pub num: i32,
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

pub fn lex() -> TokenMass {
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