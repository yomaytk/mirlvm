use super::lexer::*;
use super::*;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

pub static FRESHREGNUM: Lazy<Mutex<i32>> = Lazy::new(|| Mutex::new(0));

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
    TypeTuple(Vec<VarType>),
}

impl VarType {
    pub fn stacksize(&self) -> i32 {
        use VarType::*;
        match self {
            Word | Ptr2Word => 4,
            Long | Ptr2Long => 8,
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
            Word => 4,
            Long | Ptr2Long | Ptr2Word => 8,
            TypeTuple(_) => {
                panic!("toregregsize error in TypeTuple.")
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Arg {
    vty: VarType,
    name: VarName,
}

#[derive(Debug)]
pub struct SsaProgram {
    pub funcs: Vec<SsaFunction>,
}

impl SsaProgram {
    pub fn new(funcs: Vec<SsaFunction>) -> Self {
        Self { funcs }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SsaFunction {
    pub name: &'static str,
    pub retty: VarType,
    pub args: Vec<Var>,
    pub bls: Vec<SsaBlock>,
}

impl SsaFunction {
    pub fn new(name: &'static str, retty: VarType, args: Vec<Var>, bls: Vec<SsaBlock>) -> Self {
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
        Self { name, ty, freshnum }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SsaBlock {
    pub lb: Label,
    pub instrs: Vec<SsaInstr>,
}

impl SsaBlock {
    pub fn new(lb: Label, instrs: Vec<SsaInstr>) -> Self {
        Self { lb, instrs }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum FirstClassObj {
    Variable(Var),
    Num(i32),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CompOp {
    Ceqw,
    Csltw,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SsaInstr {
    Ret(FirstClassObj),
    Assign(ValueType, Var, Box<SsaInstr>),
    Alloc4(Var, i32),
    Storew(FirstClassObj, Var),
    Loadw(Var),
    Bop(Binop, FirstClassObj, FirstClassObj),
    Call(VarType, Label, Vec<FirstClassObj>),
    Comp(CompOp, Var, Var, FirstClassObj),
    Jnz(Var, Label, Label),
    Jmp(Label),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ValueType {
    Word,
    Long,
}

impl ValueType {
    pub fn bytesize(self) -> i32 {
        match self {
            ValueType::Word => 4,
            ValueType::Long => 8,
        }
    }
    pub fn bitsize(self) -> i32 {
        match self {
            ValueType::Word => 32,
            ValueType::Long => 64,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Environment<T: Eq + std::hash::Hash + std::fmt::Debug, U: Clone + std::fmt::Debug> {
    vars: HashMap<T, U>,
}

impl<T: Eq + std::hash::Hash + std::fmt::Debug, U: Clone + std::fmt::Debug> Environment<T, U> {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }
    pub fn get(&self, key: &T) -> U {
        // self.vars.get(key).unwrap().clone()
        let r = self.vars.get(key);
        if let Some(v) = r {
            return v.clone();
        } else {
            panic!("{:?}\n{:?}", key, self.vars);
        }
    }
    fn append(&mut self, key: T, value: U) {
        self.vars.insert(key, value);
    }
}

// parser rhs of instr
fn parseinstrrhs(
    tmass: &mut TokenMass,
    varenv: &mut Environment<&'static str, Var>,
    funenv: &mut Environment<&'static str, VarType>,
) -> SsaInstr {
    // loadw
    if tmass.eq_tkty(TokenType::Loadw) {
        let rhs = tmass.getvar_n(varenv);
        assert!(rhs.ty == VarType::Ptr2Word || rhs.ty == VarType::Ptr2Long);
        return SsaInstr::Loadw(rhs);
    }
    // binop
    if let Some(binop) = tmass.getbinop() {
        let lhs = tmass.getfirstclassobj_n(varenv);
        tmass.assert_tkty(TokenType::Comma);
        let rhs = tmass.getfirstclassobj_n(varenv);
        return SsaInstr::Bop(binop, lhs, rhs);
    }
    // call
    if tmass.eq_tkty(TokenType::Call) {
        tmass.assert_tkty(TokenType::Dollar);
        let funlb = tmass.gettext_n();
        let retty = funenv.get(&funlb);
        // arguments
        let mut args = vec![];
        tmass.assert_tkty(TokenType::Lbrace);
        if tmass.eq_tkty(TokenType::Rbrace) {
            return SsaInstr::Call(retty, funlb, args);
        }
        loop {
            if tmass.eq_tkty(TokenType::Threedot) {
                break;
            }
            let _ = tmass.gettype_n();
            let arg = tmass.getfco_n(varenv);
            args.push(arg);
            tmass.assert_tkty(TokenType::Comma);
        }
        tmass.eq_tkty(TokenType::Rbrace);
        return SsaInstr::Call(retty, funlb, args);
    }
    let curtk = tmass.getcurrent_token();
    panic!(
        "parseinstr panic {:?}: {}",
        curtk,
        &PROGRAM[curtk.poss..curtk.pose]
    );
}

fn parseinstroverall(
    tmass: &mut TokenMass,
    varenv: &mut Environment<&'static str, Var>,
    funenv: &mut Environment<&'static str, VarType>,
) -> SsaInstr {
    // ret
    if tmass.eq_tkty(TokenType::Ret) {
        let retnum = tmass.getfirstclassobj_n(varenv);
        return SsaInstr::Ret(retnum);
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
            return SsaInstr::Alloc4(var, rhs);
        }
        // ceqw, csltw
        let ctkty = tmass.cur_tkty();
        if ctkty == TokenType::Ceqw || ctkty == TokenType::Csltw {
            tmass.cpos += 1;
            let lhs = tmass.getvar_n(varenv);
            tmass.assert_tkty(TokenType::Comma);
            let rhs = tmass.getfco_n(varenv);
            var.ty = VarType::Word;
            varenv.append(var.name, var.clone());
            return if ctkty == TokenType::Ceqw {
                SsaInstr::Comp(CompOp::Ceqw, var, lhs, rhs)
            } else {
                SsaInstr::Comp(CompOp::Csltw, var, lhs, rhs)
            };
        }
        let rhs = parseinstrrhs(tmass, varenv, funenv);
        varenv.append(var.name, var.clone());
        return SsaInstr::Assign(assignty, var, Box::new(rhs));
    }
    // storew
    if tmass.eq_tkty(TokenType::Storew) {
        let lhs = tmass.getfirstclassobj_n(varenv);
        tmass.assert_tkty(TokenType::Comma);
        let rhs = tmass.getvar_n(varenv);
        return SsaInstr::Storew(lhs, rhs);
    }
    // jnz
    if tmass.eq_tkty(TokenType::Jnz) {
        let condvar = tmass.getvar_n(varenv);
        tmass.assert_tkty(TokenType::Comma);
        let blb1 = tmass.getblocklb_n();
        tmass.assert_tkty(TokenType::Comma);
        let blb2 = tmass.getblocklb_n();
        return SsaInstr::Jnz(condvar, blb1, blb2);
    }
    // jmp
    if tmass.eq_tkty(TokenType::Jmp) {
        let blb = tmass.getblocklb_n();
        return SsaInstr::Jmp(blb);
    }
    panic!("parseinstroverall error. {:?}", tmass.getcurrent_token());
}

// parse basic block
fn parsebb(
    tmass: &mut TokenMass,
    varenv: &mut Environment<&'static str, Var>,
    funenv: &mut Environment<&'static str, VarType>,
) -> SsaBlock {
    let blocklb = tmass.gettext_n();
    tmass.assert_tkty(TokenType::Colon);
    let mut instrs = vec![];
    loop {
        let tkty = tmass.cur_tkty();
        if tkty == TokenType::Blocklb || tkty == TokenType::Crbrace {
            break;
        }
        instrs.push(parseinstroverall(tmass, varenv, funenv));
    }
    SsaBlock::new(blocklb, instrs)
}

fn parseargs(tmass: &mut TokenMass, varenv: &mut Environment<&'static str, Var>) -> Vec<Var> {
    let mut argvars = vec![];
    tmass.assert_tkty(TokenType::Lbrace);
    if tmass.eq_tkty(TokenType::Rbrace) {
        return vec![];
    }
    // parse each arguments
    let mut freshnum = -1;
    loop {
        let vty = tmass.gettype_n();
        let lb = tmass.gettext_n();
        let var = Var::new(lb, vty, freshnum);
        varenv.append(lb, var.clone());
        argvars.push(var);
        if tmass.eq_tkty(TokenType::Rbrace) {
            break;
        }
        tmass.assert_tkty(TokenType::Comma);
        freshnum -= 1;
    }
    argvars
}

// parse function ...
fn parsefun(tmass: &mut TokenMass, funenv: &mut Environment<&'static str, VarType>) -> SsaFunction {
    tmass.assert_tkty(TokenType::Function);
    let functy = tmass.gettype_n();
    tmass.assert_tkty(TokenType::Dollar);
    let funclb = tmass.gettext_n();
    let mut varenv = Environment::new();
    // parse arguments
    let argvars = parseargs(tmass, &mut varenv);
    // function body
    tmass.assert_tkty(TokenType::Clbrace);
    let mut blocks = vec![];
    loop {
        let ctkty = tmass.cur_tkty();
        if ctkty == TokenType::Blocklb {
            let bblock = parsebb(tmass, &mut varenv, funenv);
            blocks.push(bblock);
        } else {
            tmass.assert_tkty(TokenType::Crbrace);
            break;
        }
    }
    SsaFunction::new(funclb, functy, argvars, blocks)
}

pub fn parse(tmass: &mut TokenMass) -> SsaProgram {
    let mut funcs = vec![];
    let mut funenv = Environment::new();
    loop {
        if tmass.cur_tkty() == TokenType::Function {
            let (funlb, retty) = tmass.getfuncdata();
            funenv.append(funlb, retty);
            let func = parsefun(tmass, &mut funenv);
            funcs.push(func);
            continue;
        }
        tmass.assert_tkty(TokenType::Eof);
        break;
    }
    SsaProgram::new(funcs)
}
