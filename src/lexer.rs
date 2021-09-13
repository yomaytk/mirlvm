use super::parser::{Env, FirstClassObj, ValueType, Var, VarType};
use super::*;

pub static RESERVEDWORDS: &[(&str, TokenType)] = &[
    ("function", TokenType::Function),
    ("w", TokenType::Word),
    ("ret", TokenType::Ret),
    ("alloc4", TokenType::Alloc4),
    ("storew", TokenType::Storew),
    ("loadw", TokenType::Loadw),
    ("add", TokenType::Bop(Binop::Add)),
    ("sub", TokenType::Bop(Binop::Sub)),
    ("call", TokenType::Call),
    ("ceqw", TokenType::Ceqw),
    ("csltw", TokenType::Csltw),
    ("jnz", TokenType::Jnz),
    ("jmp", TokenType::Jmp),
    ("phi", TokenType::Phi),
    ("data", TokenType::Data),
    ("align", TokenType::Align),
];

pub static SIGNALS: &[(&str, TokenType)] = &[
    ("(", TokenType::Lbrace),
    (")", TokenType::Rbrace),
    ("{", TokenType::Clbrace),
    ("}", TokenType::Crbrace),
    ("$", TokenType::Dollar),
    (":", TokenType::Colon),
    ("=w", TokenType::Eqw),
    ("=l", TokenType::Eql),
    ("=", TokenType::Eq),
    (",", TokenType::Comma),
    ("...", TokenType::Threedot),
    ("#", TokenType::Hash),
    (">", TokenType::Rturbo),
    ("<", TokenType::Lturbo),
    (";", TokenType::Semi),
    ("!", TokenType::Excla),
];

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Binop {
    Add,
    Sub,
}

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
    Blocklb,
    Alloc4,
    Eql,
    Eqw,
    Eq,
    Bop(Binop),
    Storew,
    Loadw,
    Comma,
    Threedot,
    Call,
    Ceqw,
    Csltw,
    Jnz,
    Jmp,
    Hash,
    Phi,
    Rturbo,
    Lturbo,
    Semi,
    Data,
    Excla,
    Align,
    String,
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
    pub fn as_tkty(&mut self, tty: TokenType) {
        assert_eq!(self.tks[self.cpos].tty, tty);
        self.cpos += 1;
    }
    pub fn cur_tkty(&self) -> TokenType {
        self.tks[self.cpos].tty
    }
    pub fn eq_tkty(&mut self, tk: TokenType) -> bool {
        let tty = self.cur_tkty();
        if tty == tk {
            self.cpos += 1;
            true
        } else {
            match (tty, tk) {
                (TokenType::Bop(_), TokenType::Bop(_)) => true,
                _ => false,
            }
        }
    }
    pub fn getnum_n(&mut self) -> i32 {
        assert_eq!(self.tks[self.cpos].tty, TokenType::Ilit);
        let res = self.tks[self.cpos].num;
        self.cpos += 1;
        res
    }
    pub fn getvar_n(&mut self, env: &Env) -> Var {
        let key = self.tks[self.cpos].get_text();
        let res = env.g_lvs(&key);
        self.cpos += 1;
        res
    }
    pub fn gettype_n(&mut self) -> VarType {
        let tktxt = self.tks[self.cpos].get_text();
        // There is change for room
        self.cpos += 1;
        match tktxt {
            "w" => VarType::Word,
            "l" => VarType::Long,
            "b" => VarType::Byte,
            _ => {
                panic!("TokenMass.gettype() error. {}", tktxt);
            }
        }
    }
    pub fn getvaltype_n(&mut self) -> ValueType {
        let tktxt = self.tks[self.cpos].get_text();
        self.cpos += 1;
        match tktxt {
            "w" => ValueType::Word,
            "l" => ValueType::Long,
            "b" => ValueType::Byte,
            _ => {
                panic!("getvaltype_n() error. {}", tktxt);
            }
        }
    }
    pub fn gettext_n(&mut self) -> &'static str {
        let tktext = self.tks[self.cpos].get_text();
        self.cpos += 1;
        tktext
    }
    pub fn getfco_n(&mut self, vty: VarType, env: &mut Env) -> FirstClassObj {
        let ctk = self.getcurrent_token();
        let lb = self.gettext_n();
        match ctk.tty {
            TokenType::Ident => {
                let var = env.g_lvs(&lb);
                FirstClassObj::Variable(var)
            }
            TokenType::Ilit => FirstClassObj::Num(vty, ctk.num),
            TokenType::String => FirstClassObj::String(lb),
            _ => {
                panic!("getfco_n error. {:?}", self.getcurrent_token());
            }
        }
    }
    pub fn getblocklb_n(&mut self) -> &'static str {
        let lb = &self.tks[self.cpos].get_text();
        self.eq_tkty(TokenType::Blocklb);
        lb
    }
    pub fn getcurrent_token(&self) -> Token {
        self.tks[self.cpos]
    }
    pub fn getbinop(&mut self) -> Option<Binop> {
        let tty = self.cur_tkty();
        use TokenType::*;
        match tty {
            Bop(Binop::Add) => {
                self.cpos += 1;
                Some(Binop::Add)
            }
            Bop(Binop::Sub) => {
                self.cpos += 1;
                Some(Binop::Sub)
            }
            _ => None,
        }
    }
    pub fn getfuncdata(&mut self) -> (&'static str, VarType) {
        self.cpos += 1;
        let retty;
        let back;
        if self.cur_tkty() == TokenType::Dollar {
            retty = VarType::Void;
            back = 3;
        } else {
            retty = self.gettype_n();
            back = 4;
        }
        self.as_tkty(TokenType::Dollar);
        let funlb = self.gettext_n();
        self.cpos -= back;
        (funlb, retty)
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
    let mut tms = TokenMass::new();

    loop {
        if pos == pglen {
            break;
        }
        if pgchars[pos].is_whitespace() {
            pos += 1;
            continue;
        }
        if pgchars[pos] == '#' {
            while pgchars[pos] != '\n' {
                pos += 1;
            }
            continue;
        }

        // identification or reserved words
        if pgchars[pos] == '@' || pgchars[pos] == '%' || pgchars[pos].is_ascii_alphabetic() {
            let mut pose = pos;
            let mut tty = TokenType::Ident;
            pose += 1;
            while pgchars[pose].is_ascii_alphanumeric() || pgchars[pose] == '.' {
                pose += 1;
            }
            if pgchars[pos] == '@' {
                tty = TokenType::Blocklb;
                tms.push(Token::new(tty, pos + 1, pose, -1));
                pos = pose;
                continue;
            }
            for resw in RESERVEDWORDS {
                if resw.0 == &program[pos..pose] {
                    tty = resw.1;
                    break;
                }
            }
            tms.push(Token::new(tty, pos, pose, -1));
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
            tms.push(Token::new(TokenType::Ilit, poss, pos, num));
            continue;
        }

        // string
        if pgchars[pos] == '\"' {
            let mut pose = pos + 1;
            while pgchars[pose] != '\"' {
                pose += 1;
            }
            tms.push(Token::new(TokenType::String, pos + 1, pose, -1));
            pos = pose + 1;
            continue;
        }

        // signals
        let mut nextloop = false;
        for sig in SIGNALS {
            let signal = sig.0;
            let pose = pos + signal.len();
            if signal == &program[pos..pose] {
                let tty = sig.1;
                nextloop = true;
                tms.push(Token::new(tty, pos, pose, -1));
                pos = pose;
                break;
            }
        }
        if nextloop {
            continue;
        }

        panic!("failed lex program. next letter = {}", pgchars[pos]);
    }
    tms.push(Token::new(TokenType::Eof, 0, 0, -1));
    tms
}
