use super::lexer::*;
use super::*;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

pub static FRESHREGNUM: Lazy<Mutex<i32>> = Lazy::new(|| Mutex::new(0));
static GFRSN: Lazy<Mutex<i32>> = Lazy::new(|| Mutex::new(-1));

fn ggfrsn() -> i32 {
    let cgf = *GFRSN.lock().unwrap();
    *GFRSN.lock().unwrap() = cgf - 1;
    cgf
}

pub fn nextfreshregister() -> i32 {
    let res = *FRESHREGNUM.lock().unwrap();
    *FRESHREGNUM.lock().unwrap() += 1;
    res
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ValueType {
    Word,
    Long,
    Byte,
    Z,
}

impl ValueType {
    pub fn bytesize(self) -> i32 {
        use ValueType::*;
        match self {
            Word => 4,
            Long => 8,
            Byte => 1,
            Z => -1,
        }
    }
    pub fn bitsize(self) -> i32 {
        use ValueType::*;
        match self {
            Word => 32,
            Long => 64,
            Byte => 8,
            Z => -1,
        }
    }
    pub fn tovarty(&self) -> VarType {
        use ValueType::*;
        match self {
            Word => VarType::Word,
            Long => VarType::Long,
            Byte => VarType::Byte,
            Z => {
                panic!("tovarty error: {:?}", self);
            }
        }
    }
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
    ConT(Vec<(VarType, u32)>),
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
            ConT(cont) => {
                let mut size = 0;
                for t in cont {
                    size += t.0.stacksize() * t.1 as i32;
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
            Byte => 1,
            Void => 0,
            TypeTuple(_) => {
                panic!("toregregsize error in TypeTuple.")
            }
            ConT(cont) => {
                let mut size = 0;
                for t in cont {
                    size += t.0.stacksize() * t.1 as i32;
                }
                size
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
    pub gvs: Vec<Gdata>,
}

impl SsaProgram {
    pub fn new(funcs: Vec<SsaFunction>, gvs: Vec<Gdata>) -> Self {
        Self { funcs, gvs }
    }
}

#[derive(Debug, Clone)]
pub struct Gdata {
    pub frsn: i32,
    pub al: i32,
    pub lb: &'static str,
    pub dts: Vec<FirstClassObj>,
    pub types: VarType,
}

impl Gdata {
    pub fn new(
        frsn: i32,
        al: i32,
        lb: &'static str,
        dts: Vec<FirstClassObj>,
        types: VarType,
    ) -> Self {
        Self {
            frsn,
            al,
            lb,
            dts,
            types,
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
    Num(VarType, i32),
    String(&'static str),
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

#[derive(Debug, Clone)]
pub struct Env {
    fns: HashMap<&'static str, VarType>,
    lvs: HashMap<&'static str, Var>,
    gvs: HashMap<&'static str, Gdata>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            fns: HashMap::new(),
            lvs: HashMap::new(),
            gvs: HashMap::new(),
        }
    }
    pub fn g_fns(&self, key: &'static str) -> VarType {
        if let Some(v) = self.fns.get(key) {
            return v.clone();
        } else {
            return VarType::Void;
        }
    }
    pub fn g_lvs(&self, key: &'static str) -> Var {
        if let Some(v) = self.lvs.get(key) {
            return v.clone();
        } else {
            self.g_gvs(key)
        }
    }
    pub fn g_gvs(&self, key: &'static str) -> Var {
        if let Some(v) = self.gvs.get(key) {
            return Var::new(v.lb, v.types.clone(), -10);
        } else {
            panic!("{} is not in Env.\nEnv: {:?}", key, self);
        }
    }
    fn i_fns(&mut self, key: &'static str, vty: VarType) {
        self.fns.insert(key, vty);
    }
    fn i_lvs(&mut self, key: &'static str, var: Var) {
        self.lvs.insert(key, var);
    }
    fn i_gvs(&mut self, key: &'static str, ssd: Gdata) {
        self.gvs.insert(key, ssd);
    }
}

// parser rhs of instr
fn parseinstrrhs(tms: &mut TokenMass, env: &mut Env) -> SsaInstr {
    // loadw
    if tms.eq_tkty(TokenType::Loadw) {
        let rhs = tms.getvar_n(env);
        assert!(rhs.ty == VarType::Ptr2Word || rhs.ty == VarType::Ptr2Long);
        return SsaInstr::new(SsaInstrOp::Loadw(rhs));
    }
    // binop
    if let Some(binop) = tms.getbinop() {
        let lhs = tms.getfco_n(VarType::Word, env);
        tms.as_tkty(TokenType::Comma);
        let rhs = tms.getfco_n(VarType::Word, env);
        return SsaInstr::new(SsaInstrOp::Bop(binop, lhs, rhs));
    }
    // call
    if tms.eq_tkty(TokenType::Call) {
        tms.as_tkty(TokenType::Dollar);
        let funlb = tms.gettext_n();
        let retty = env.g_fns(&funlb);
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
                let ty = tms.gettype_n();
                tms.eq_tkty(TokenType::Dollar);
                let mut arg = tms.getfco_n(ty, env);

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
            let fco = tms.getfco_n(VarType::Word, env);
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

fn parseinstroverall(tms: &mut TokenMass, env: &mut Env) -> SsaInstr {
    // ret
    if tms.eq_tkty(TokenType::Ret) {
        let retnum = tms.getfco_n(VarType::Word, env);
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
            env.i_lvs(var.name, var.clone());
            return SsaInstr::new(SsaInstrOp::Alloc4(var, rhs));
        }
        // ceqw, csltw
        let ctkty = tms.cur_tkty();
        if ctkty == TokenType::Ceqw || ctkty == TokenType::Csltw {
            tms.cpos += 1;
            let lhs = tms.getvar_n(env);
            tms.as_tkty(TokenType::Comma);
            let rhs = tms.getfco_n(VarType::Word, env);
            var.ty = VarType::Word;
            env.i_lvs(var.name, var.clone());
            return if ctkty == TokenType::Ceqw {
                SsaInstr::new(SsaInstrOp::Comp(CompOp::Ceqw, var, lhs, rhs))
            } else {
                SsaInstr::new(SsaInstrOp::Comp(CompOp::Csltw, var, lhs, rhs))
            };
        }
        let rhs = parseinstrrhs(tms, env);
        env.i_lvs(var.name, var.clone());
        return SsaInstr::new(SsaInstrOp::Assign(assignty, var, Box::new(rhs)));
    }
    // storew
    if tms.eq_tkty(TokenType::Storew) {
        let lhs = tms.getfco_n(VarType::Word, env);
        tms.as_tkty(TokenType::Comma);
        let rhs = tms.getvar_n(env);
        return SsaInstr::new(SsaInstrOp::Storew(lhs, rhs));
    }
    // jnz
    if tms.eq_tkty(TokenType::Jnz) {
        let condvar = tms.getvar_n(env);
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
    // call
    if tms.cur_tkty() == TokenType::Call {
        return parseinstrrhs(tms, env);
    }
    panic!("parseinstroverall error. {:?}", tms.getcurrent_token());
}

// parse basic block
fn parsebb(tms: &mut TokenMass, env: &mut Env) -> SsaBlock {
    let mut ssb = SsaBlock::new(tms.gettext_n(), vec![]);
    tms.as_tkty(TokenType::Colon);
    loop {
        let tkty = tms.cur_tkty();
        if tkty == TokenType::Blocklb || tkty == TokenType::Crbrace {
            break;
        }
        ssb.instrs.push(parseinstroverall(tms, env));
    }
    ssb
}

fn parseargs(tms: &mut TokenMass, env: &mut Env) -> Vec<Var> {
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
        env.i_lvs(lb, var.clone());
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
fn parsefun(tms: &mut TokenMass, env: &mut Env) -> SsaFunction {
    let mut sfn = SsaFunction::new("", VarType::Void, vec![], vec![]);
    if tms.cur_tkty() != TokenType::Dollar {
        sfn.retty = tms.gettype_n();
    }
    tms.as_tkty(TokenType::Dollar);
    sfn.name = tms.gettext_n();
    // parse arguments
    sfn.args = parseargs(tms, env);
    // function body
    tms.as_tkty(TokenType::Clbrace);
    loop {
        let ctkty = tms.cur_tkty();
        if ctkty == TokenType::Blocklb {
            sfn.bls.push(parsebb(tms, env));
        } else {
            tms.as_tkty(TokenType::Crbrace);
            break;
        }
    }
    sfn
}

fn parsedata(tms: &mut TokenMass, env: &mut Env) -> Gdata {
    let mut gd = Gdata::new(ggfrsn(), 0, "", vec![], VarType::Void);
    tms.as_tkty(TokenType::Dollar);
    gd.lb = tms.gettext_n();
    tms.as_tkty(TokenType::Eq);
    if tms.eq_tkty(TokenType::Align) {
        gd.al = tms.getnum_n();
    }
    tms.as_tkty(TokenType::Clbrace);
    let mut typesv = vec![];

    // get each element from global data
    loop {
        let dty = tms.gettype_n();
        let mut cnt = 0;
        while !tms.eq_tkty(TokenType::Comma) {
            gd.dts.push(tms.getfco_n(dty.clone(), env));
            cnt += 1;
            if tms.eq_tkty(TokenType::Crbrace) {
                gd.types = VarType::ConT(typesv);
                env.i_gvs(gd.lb, gd.clone());
                return gd;
            }
        }
        typesv.push((dty, cnt));
    }
}

pub fn parse(tms: &mut TokenMass) -> SsaProgram {
    let mut spg = SsaProgram::new(vec![], vec![]);
    let mut env = Env::new();
    loop {
        // function
        if tms.eq_tkty(TokenType::Function) {
            let (funlb, retty) = tms.getfuncdata();
            env.i_fns(funlb, retty);
            spg.funcs.push(parsefun(tms, &mut env));
            continue;
        }
        // global data
        if tms.eq_tkty(TokenType::Data) {
            let ssd = parsedata(tms, &mut env);
            env.i_gvs(ssd.lb, ssd.clone());
            spg.gvs.push(ssd);
            continue;
        }
        tms.as_tkty(TokenType::Eof);
        break;
    }
    spg
}
