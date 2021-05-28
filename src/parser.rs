use std::collections::HashMap;
use super::*;
use super::lexer::*;
use once_cell::sync::Lazy;
use std::sync::Mutex;

pub static FRESHREGNUM: Lazy<Mutex<i32>> = Lazy::new(|| {
    Mutex::new(0)
});

pub fn nextfreshregister() -> i32 {
    let res = *FRESHREGNUM.lock().unwrap();
    *FRESHREGNUM.lock().unwrap() += 1;
    res
}

#[derive(Clone, Debug, PartialEq)]
pub enum VarType {
    Word,
    Long,
    Ptr2Word,
    Ptr2Long,
    TypeTuple(Vec<VarType>)
}

impl VarType {
    pub fn stacksize(&self) -> i32 {
        use VarType::*;
        match self {
            Word | Ptr2Word => { 4 }
            Long | Ptr2Long => { 8 }
            TypeTuple(vvt) => {
                let mut size = 0;
                for v in vvt {
                    size += v.stacksize()
                }
                size
            }
        }
    }
    pub fn toregrefsize(&self) -> i32 {
        use VarType::*;
        match self {
            Word => { 4 }
            Long | Ptr2Long | Ptr2Word => { 8 }
            TypeTuple(_) => { panic!("toregregsize error in TypeTuple.") }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Arg {
    vty: VarType,
    name: VarName,
}

#[derive(Debug)]
pub struct ParserProgram {
    pub funcs: Vec<ParserFunction>,
}

impl ParserProgram {
    pub fn new(funcs: Vec<ParserFunction>) -> Self {
        Self {
            funcs,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ParserFunction {
    pub name: Label,
    pub retty: VarType,
    pub args: Vec<Var>,
    pub bls: Vec<ParserBlock>,
}

impl ParserFunction {
    pub fn new(name: String, retty: VarType, args: Vec<Var>, bls: Vec<ParserBlock>) -> Self {
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
    pub freshnum: i32,
}

impl Var {
    pub fn new(name: &'static str, ty: VarType, freshnum: i32) -> Self {
        Self {
            name,
            ty,
            freshnum,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ParserBlock {
    pub lb: Label,
    pub instrs: Vec<ParserInstr>
}

impl ParserBlock {
    pub fn new(lb: String, instrs: Vec<ParserInstr>) -> Self {
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
pub enum ParserInstr {
    Ret(FirstClassObj),
    Assign(ValueType, Var, Box<ParserInstr>),
    Alloc4(Var, i32),
    Storew(FirstClassObj, Var),
    Loadw(Var),
    Add(FirstClassObj, FirstClassObj),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ValueType {
    Word,
    Long,
}

impl ValueType {
    pub fn bytesize(self) -> i32 {
        match self {
            ValueType::Word => { 4 }
            ValueType::Long => { 8 }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct VariableEnvironment<T: Eq + std::hash::Hash + std::fmt::Debug, U: Clone> {
    vars: HashMap<T, U>,
}

impl<T: Eq + std::hash::Hash + std::fmt::Debug, U: Clone> VariableEnvironment<T, U> {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new()
        }
    }
    pub fn get(&self, key: &T) -> U {
        // self.vars.get(key).unwrap().clone()
        let r = self.vars.get(key);
        if let Some(v) = r {
            return v.clone();
        } else {
            panic!("{:?}", key);
        }
    }
    fn append(&mut self, key: T, value: U) {
        self.vars.insert(key, value);
    }
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
fn parseinstrrhs(tmass: &mut TokenMass, varenv: &mut VariableEnvironment<&'static str, Var>) -> ParserInstr {
    // loadw
    if tmass.eq_tkty(TokenType::Loadw) {
        let rhs = tmass.getvar_n(varenv);
        assert!(rhs.ty == VarType::Ptr2Word || rhs.ty == VarType::Ptr2Long);
        return ParserInstr::Loadw(rhs);
    }
    // add
    if tmass.eq_tkty(TokenType::Add) {
        let lhs = tmass.getfirstclassobj_n(varenv);
        tmass.assert_tkty(TokenType::Comma);
        let rhs = tmass.getfirstclassobj_n(varenv);
        return ParserInstr::Add(lhs, rhs);
    }
    let curtk = tmass.getcurrent_token();
    panic!("parseinstr panic {:?}: {}", curtk, &PROGRAM[curtk.poss..curtk.pose]);
}

fn parseinstroverall(tmass: &mut TokenMass, varenv: &mut VariableEnvironment<&'static str, Var>) -> ParserInstr {
    // ret
    if tmass.eq_tkty(TokenType::Ret) {
        let retnum = tmass.getfirstclassobj_n(varenv);
        return ParserInstr::Ret(retnum)
    }
    // lhs =* rhs instruction
    if tmass.cur_tkty() == TokenType::Ident {
        let varname = tmass.gettext_n();
        let cur_tkty = tmass.cur_tkty();
        let assignty;
        let mut var;
        if cur_tkty == TokenType::Eql { 
            assignty = ValueType::Long;
            var = Var::new(varname, VarType::Long, nextfreshregister());
        } else {
            assert_eq!(cur_tkty, TokenType::Eqw);
            assignty = ValueType::Word;
            var = Var::new(varname, VarType::Word, nextfreshregister());
        }
        tmass.cpos += 1;
        // alloc4
        if tmass.eq_tkty(TokenType::Alloc4) {
            let rhs = tmass.getnum_n();
            var.ty = VarType::Ptr2Word;
            varenv.append(var.name, var.clone());
            return ParserInstr::Alloc4(var, rhs);
        }
        let rhs = parseinstrrhs(tmass, varenv);
        varenv.append(var.name, var.clone());
        return ParserInstr::Assign(assignty, var, Box::new(rhs));
    }
    // storew
    if tmass.eq_tkty(TokenType::Storew) {
        let lhs = tmass.getfirstclassobj_n(varenv);
        tmass.assert_tkty(TokenType::Comma);
        let rhs = tmass.getvar_n(varenv);
        return ParserInstr::Storew(lhs, rhs);
    }
    panic!("parseinstroverall error. {:?}", tmass.getcurrent_token());
}

// parse basic block
fn parsebb(tmass: &mut TokenMass) -> ParserBlock {
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
    }
    ParserBlock::new(String::from(blocklb), instrs)
}

// parse function ...
fn parsefun(tmass: &mut TokenMass) -> ParserFunction {
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
    ParserFunction::new(String::from(funclb), functy, argvars, blocks)
}

pub fn parse(tmass: &mut TokenMass) -> ParserProgram {
    let mut funcs = vec![];
    loop {
        if tmass.cur_tkty() == TokenType::Function {
            funcs.push(parsefun(tmass));
            continue;
        }
        tmass.assert_tkty(TokenType::Eof);
        break;
    }
    ParserProgram::new(funcs)
}