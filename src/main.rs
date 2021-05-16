use once_cell::sync::Lazy;
use std::env;
use std::fs;
use std::sync::Mutex;

const RESERVED_SIZE: usize = 3;
const SIGNALS_SIZE: usize = 5;

pub static RESERVEDWORDS: [(&str, TokenType); RESERVED_SIZE] = [
    ("function", TokenType::Function),
    ("w", TokenType::Word),
    ("ret", TokenType::Ret),
];
pub static SIGNALS: [(&str, TokenType); SIGNALS_SIZE] = [
    ("(", TokenType::Lbrace),
    (")", TokenType::Rbrace),
    ("{", TokenType::Crbrace),
    ("}", TokenType::Clbrace),
    ("$", TokenType::Dollar),
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
    Dummy,
}

#[derive(Debug)]
struct TokenMass {
    tmass: Vec<Token>,
}

impl TokenMass {
    pub fn new() -> Self {
        Self {
            tmass: vec![]
        }
    }
    pub fn push(&mut self, tk: Token) {
        self.tmass.push(tk)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Token {
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
}

pub static PROGRAM: Lazy<Mutex<String>> = Lazy::new(|| {
    let file: String = env::args().collect::<Vec<String>>().last().unwrap().clone();
    Mutex::new(fs::read_to_string(file).expect("failed to read file."))
});

fn lex() -> TokenMass {
    let program = PROGRAM.lock().unwrap().clone();
    let pgchars: Vec<char> = (&program[..]).chars().collect();
    let pglen = program.len();
    let mut pos = 0;
    let mut tmass = TokenMass::new();
    println!("{}", program);
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
            let pose = pos+1;
            if SIGNALS[i].0 == &program[pos..pose] {
                let tty = SIGNALS[i].1;
                nextloop = true;
                tmass.push(Token::new(tty, pos, pose, -1));
                pos = pose;
                break;
            }
        }
        if nextloop { continue; }

        panic!("failed lex program.");
    }
    tmass
}

fn main() {
    let tmass = lex();
    println!("{:#?}", tmass);
}
