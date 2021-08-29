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
    Byte,
    Ptr2Word,
    Ptr2Long,
    TypeTuple(Vec<VarType>),
    Void,
}

impl VarType {
    pub fn stacksize(&self) -> i32 {
        use VarType::*;
        match self {
            Word | Ptr2Word => 4,
            Long | Ptr2Long => 8,
            Byte => 1,
            TypeTuple(vvt) => {
                let mut size = 0;
                for v in vvt {
                    size += v.stacksize()
                }
                size
            }
            Void => 1,
        }
    }
    pub fn toregrefsize(&self) -> i32 {
        use VarType::*;
        match self {
            Word => 4,
            Long | Ptr2Long | Ptr2Word => 8,
            Byte => 1,
            Void => 0,
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

pub struct SsaData {
    pub al: i32,
    pub lb: &'static str,
    pub dts: Vec<FirstClassObj>,
}

impl SsaData {
    pub fn new(al: i32, lb: &'static str, dts: Vec<FirstClassObj>) -> Self {
        Self {
            al,
            lb,
            dts,
        }
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
    pub frsn: i32,
}

impl Var {
    pub fn new(name: &'static str, ty: VarType, frsn: i32) -> Self {
        Self { name, ty, frsn }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SsaBlock {
    pub lb: Label,
    pub instrs: Vec<SsaInstr>,
}

impl SsaBlock {
    pub fn new(lb: Label, instrs: Vec<SsaInstr>) -> Self {
        Self {
            lb: lb,
            instrs: instrs,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum FirstClassObj {
    Variable(Var),
    Num(ValueType, i32),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CompOp {
    Ceqw,
    Csltw,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SsaInstrOp {
    Ret(FirstClassObj),
    Assign(ValueType, Var, Box<SsaInstr>),
    Alloc4(Var, i32),
    Storew(FirstClassObj, Var),
    Loadw(Var),
    Bop(Binop, FirstClassObj, FirstClassObj),
    Call(VarType, Label, Vec<FirstClassObj>, bool),
    Comp(CompOp, Var, Var, FirstClassObj),
    Jnz(Var, Label, Label),
    Jmp(Label),
    Phi(Vec<(Label, FirstClassObj)>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct SsaInstr {
    pub op: SsaInstrOp,
    pub living: bool,
    pub bblb: Label,
}

impl SsaInstr {
    fn new(op: SsaInstrOp) -> Self {
        Self {
            op,
            living: false,
            bblb: "",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ValueType {
    Word,
    Long,
    Byte,
}

impl ValueType {
    pub fn bytesize(self) -> i32 {
        match self {
            ValueType::Word => 4,
            ValueType::Long => 8,
            ValueType::Byte => 1,
        }
    }
    pub fn bitsize(self) -> i32 {
        match self {
            ValueType::Word => 32,
            ValueType::Long => 64,
            ValueType::Byte => 8,
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
    tms: &mut TokenMass,
    varenv: &mut Environment<&'static str, Var>,
    funenv: &mut Environment<&'static str, VarType>,
) -> SsaInstr {
    // loadw
    if tms.eq_tkty(TokenType::Loadw) {
        let rhs = tms.getvar_n(varenv);
        assert!(rhs.ty == VarType::Ptr2Word || rhs.ty == VarType::Ptr2Long);
        return SsaInstr::new(SsaInstrOp::Loadw(rhs));
    }
    // binop
    if let Some(binop) = tms.getbinop() {
        let lhs = tms.getfco_n(Some(ValueType::Word), varenv);
        tms.as_tkty(TokenType::Comma);
        let rhs = tms.getfco_n(Some(ValueType::Word), varenv);
        return SsaInstr::new(SsaInstrOp::Bop(binop, lhs, rhs));
    }
    // call
    if tms.eq_tkty(TokenType::Call) {
        tms.as_tkty(TokenType::Dollar);
        let funlb = tms.gettext_n();
        let retty = funenv.get(&funlb);
        let mut variadic = false;
        // arguments
        let mut args = vec![];
        tms.as_tkty(TokenType::Lbrace);
        if tms.eq_tkty(TokenType::Rbrace) {
            return SsaInstr::new(SsaInstrOp::Call(retty, funlb, args, variadic));    
        }
        loop {
            if tms.eq_tkty(TokenType::Threedot) {
                variadic = true;
            } else {
                let _ = tms.gettype_n();
                let arg = tms.getfco_n(Some(ValueType::Word), varenv);
                args.push(arg);
            }
            if tms.eq_tkty(TokenType::Rbrace) {
                break;
            }
            tms.as_tkty(TokenType::Comma);
        }
        return SsaInstr::new(SsaInstrOp::Call(retty, funlb, args, variadic));
    }
    if tms.eq_tkty(TokenType::Phi) {
        let mut pv = vec![];
        while tms.cur_tkty() == TokenType::Blocklb {
            let lb = tms.gettext_n();
            let fco = tms.getfco_n(Some(ValueType::Word), varenv);
            pv.push((lb, fco));
        }
        return SsaInstr::new(SsaInstrOp::Phi(pv));
    }
    let curtk = tms.getcurrent_token();
    panic!(
        "parseinstr panic {:?}: {}",
        curtk,
        &PROGRAM[curtk.poss..curtk.pose]
    );
}

fn parseinstroverall(
    tms: &mut TokenMass,
    varenv: &mut Environment<&'static str, Var>,
    funenv: &mut Environment<&'static str, VarType>,
) -> SsaInstr {
    // ret
    if tms.eq_tkty(TokenType::Ret) {
        let retnum = tms.getfco_n(Some(ValueType::Word), varenv);
        return SsaInstr::new(SsaInstrOp::Ret(retnum));
    }
    // lhs =* rhs instruction
    if tms.cur_tkty() == TokenType::Ident {
        let varn = tms.gettext_n();
        let cur_tkty = tms.cur_tkty();
        let assignty;
        let mut var;
        if cur_tkty == TokenType::Eql {
            assignty = ValueType::Long;
            var = Var::new(varn, VarType::Long, nextfreshregister());
        } else {
            assert_eq!(cur_tkty, TokenType::Eqw);
            assignty = ValueType::Word;
            var = Var::new(varn, VarType::Word, nextfreshregister());
        }
        tms.cpos += 1;
        // alloc4
        if tms.eq_tkty(TokenType::Alloc4) {
            let rhs = tms.getnum_n();
            var.ty = VarType::Ptr2Word;
            varenv.append(var.name, var.clone());
            return SsaInstr::new(SsaInstrOp::Alloc4(var, rhs));
        }
        // ceqw, csltw
        let ctkty = tms.cur_tkty();
        if ctkty == TokenType::Ceqw || ctkty == TokenType::Csltw {
            tms.cpos += 1;
            let lhs = tms.getvar_n(varenv);
            tms.as_tkty(TokenType::Comma);
            let rhs = tms.getfco_n(Some(ValueType::Word), varenv);
            var.ty = VarType::Word;
            varenv.append(var.name, var.clone());
            return if ctkty == TokenType::Ceqw {
                SsaInstr::new(SsaInstrOp::Comp(CompOp::Ceqw, var, lhs, rhs))
            } else {
                SsaInstr::new(SsaInstrOp::Comp(CompOp::Csltw, var, lhs, rhs))
            };
        }
        let rhs = parseinstrrhs(tms, varenv, funenv);
        varenv.append(var.name, var.clone());
        return SsaInstr::new(SsaInstrOp::Assign(assignty, var, Box::new(rhs)));
    }
    // storew
    if tms.eq_tkty(TokenType::Storew) {
        let lhs = tms.getfco_n(Some(ValueType::Word), varenv);
        tms.as_tkty(TokenType::Comma);
        let rhs = tms.getvar_n(varenv);
        return SsaInstr::new(SsaInstrOp::Storew(lhs, rhs));
    }
    // jnz
    if tms.eq_tkty(TokenType::Jnz) {
        let condvar = tms.getvar_n(varenv);
        tms.as_tkty(TokenType::Comma);
        let blb1 = tms.getblocklb_n();
        tms.as_tkty(TokenType::Comma);
        let blb2 = tms.getblocklb_n();
        return SsaInstr::new(SsaInstrOp::Jnz(condvar, blb1, blb2));
    }
    // jmp
    if tms.eq_tkty(TokenType::Jmp) {
        let blb = tms.getblocklb_n();
        return SsaInstr::new(SsaInstrOp::Jmp(blb));
    }
    panic!("parseinstroverall error. {:?}", tms.getcurrent_token());
}

// parse basic block
fn parsebb(
    tms: &mut TokenMass,
    varenv: &mut Environment<&'static str, Var>,
    funenv: &mut Environment<&'static str, VarType>,
) -> SsaBlock {
    let blocklb = tms.gettext_n();
    tms.as_tkty(TokenType::Colon);
    let mut instrs = vec![];
    loop {
        let tkty = tms.cur_tkty();
        if tkty == TokenType::Blocklb || tkty == TokenType::Crbrace {
            break;
        }
        instrs.push(parseinstroverall(tms, varenv, funenv));
    }
    SsaBlock::new(blocklb, instrs)
}

fn parseargs(tms: &mut TokenMass, varenv: &mut Environment<&'static str, Var>) -> Vec<Var> {
    let mut argvars = vec![];
    tms.as_tkty(TokenType::Lbrace);
    if tms.eq_tkty(TokenType::Rbrace) {
        return vec![];
    }
    // parse each arguments
    let mut frsn = -1;
    loop {
        let vty = tms.gettype_n();
        let lb = tms.gettext_n();
        let var = Var::new(lb, vty, frsn);
        varenv.append(lb, var.clone());
        argvars.push(var);
        if tms.eq_tkty(TokenType::Rbrace) {
            break;
        }
        tms.as_tkty(TokenType::Comma);
        frsn -= 1;
    }
    argvars
}

// parse function ...
fn parsefun(tms: &mut TokenMass, funenv: &mut Environment<&'static str, VarType>) -> SsaFunction {
    tms.as_tkty(TokenType::Function);
    let mut functy = VarType::Void;
    if tms.cur_tkty() != TokenType::Dollar {
        functy = tms.gettype_n();
    }
    tms.as_tkty(TokenType::Dollar);
    let funclb = tms.gettext_n();
    let mut varenv = Environment::new();
    // parse arguments
    let argvars = parseargs(tms, &mut varenv);
    // function body
    tms.as_tkty(TokenType::Clbrace);
    let mut blocks = vec![];
    loop {
        let ctkty = tms.cur_tkty();
        if ctkty == TokenType::Blocklb {
            let bblock = parsebb(tms, &mut varenv, funenv);
            blocks.push(bblock);
        } else {
            tms.as_tkty(TokenType::Crbrace);
            break;
        }
    }
    SsaFunction::new(funclb, functy, argvars, blocks)
}

fn parsedata(tms: &mut TokenMass, varenv: &mut Environment<&'static str, Var>) -> SsaData {
    let mut gd = SsaData::new(0, "", vec![]);
    tms.as_tkty(TokenType::Dollar);
    gd.lb = tms.gettext_n();
    tms.as_tkty(TokenType::Eq);
    if tms.eq_tkty(TokenType::Align) {
        gd.al = tms.getnum_n();
    }
    tms.as_tkty(TokenType::Clbrace);
    loop {
        let dty = tms.getvaltype_n();
        while !tms.eq_tkty(TokenType::Comma) {
            gd.dts.push(tms.getfco_n(Some(dty), varenv));
            if tms.eq_tkty(TokenType::Crbrace) {
                return gd;
            }
        }
    }
}

pub fn parse(tms: &mut TokenMass) -> SsaProgram {
    let mut funcs = vec![];
    let mut funenv = Environment::new();
    loop {
        if tms.cur_tkty() == TokenType::Function {
            let (funlb, retty) = tms.getfuncdata();
            funenv.append(funlb, retty);
            let func = parsefun(tms, &mut funenv);
            funcs.push(func);
            continue;
        }
        tms.as_tkty(TokenType::Eof);
        break;
    }
    SsaProgram::new(funcs)
}
