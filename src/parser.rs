use std::collections::HashMap;
use super::*;
use super::lexer::*;

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
    pub fn get(&self, key: &'static str) -> Var {
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

pub fn parse(tmass: &mut TokenMass) -> Program {
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