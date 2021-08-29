use super::lexer::Binop;
use super::parser::*;
use super::*;
use rega::GENEREGSIZE;
use std::collections::HashMap;
use std::fmt;

pub static NULLNUMBER: i32 = -100;
pub static REGDEFASIZE: i32 = 4;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Register {
    pub vr: i32,
    pub rr: i32,
    pub btday: i32,
    pub daday: i32,
    pub regsize: i32,
}

impl Register {
    pub fn new(vr: i32) -> Self {
        Self {
            vr,
            rr: NULLNUMBER,
            btday: NULLNUMBER,
            daday: NULLNUMBER,
            regsize: REGDEFASIZE,
        }
    }
    pub fn newall(vr: i32, btday: i32, daday: i32, regsize: i32) -> Self {
        Self {
            vr,
            rr: NULLNUMBER,
            btday,
            daday,
            regsize,
        }
    }
    pub fn regalloc(&mut self, realregs: &mut [i32; GENEREGSIZE]) {
        if self.vr < 0 {
            self.rr = GENEREGSIZE as i32 - 1 + -(self.vr);
            return;
        }
        // find register already allocated
        let mut newrr = -1;
        for i in 0..GENEREGSIZE {
            if realregs[i] == self.vr {
                self.rr = i as i32;
                return;
            }
            if newrr == -1 && realregs[i] == -1 {
                newrr = i as i32;
            }
        }
        // new register allocate
        if newrr == -1 {
            panic!("Not enough register.");
        } else {
            self.rr = newrr;
            realregs[self.rr as usize] = self.vr;
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RegorNum {
    Reg(Register),
    Num(i32),
}

#[derive(Clone, Debug, PartialEq)]
pub enum LowIrInstr {
    Movenum(Register, i32),
    Movereg(Register, Register),
    Ret(Register),
    Storewreg(Register, i32),
    Storewnum(i32, i32),
    Loadw(Register, i32),
    Bop(Binop, Register, RegorNum),
    Call(Register, Label, Vec<RegorNum>, Vec<usize>),
    Comp(CompOp, Register, Register, RegorNum),
    Jnz(Register, Label, Label),
    Jmp(Label),
}

impl fmt::Display for LowIrInstr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use LowIrInstr::*;
        match self {
            Movenum(r, c) => {
                write!(f, "\tmove {}r[{}]({}), {}", r.regsize, r.vr, r.rr, c)
            }
            Movereg(r1, r2) => {
                write!(
                    f,
                    "\tmove {}r[{}]({}), {}r[{}]({})",
                    r1.regsize, r1.vr, r1.rr, r2.regsize, r2.vr, r2.rr
                )
            }
            Ret(r) => {
                write!(f, "\tret {}r[{}]({})", r.regsize, r.vr, r.rr)
            }
            Storewreg(r, offset) => {
                write!(
                    f,
                    "\tstorewreg [base-{}], {}r[{}]({})",
                    offset, r.regsize, r.vr, r.rr
                )
            }
            Storewnum(num, offset) => {
                write!(f, "\tstorewnum [base-{}], {}", offset, num)
            }
            Loadw(r, offset) => {
                write!(
                    f,
                    "\tloadw {}r[{}]({}), [base-{}]",
                    r.regsize, r.vr, r.rr, offset
                )
            }
            Bop(binop, r1, r2) => {
                let bop = match binop {
                    Binop::Add => "add",
                    Binop::Sub => "sub",
                };
                let rhs = match r2 {
                    RegorNum::Num(num) => format!("{}", num),
                    RegorNum::Reg(r) => format!("{}r[{}]({})", r.regsize, r.vr, r.rr),
                };
                write!(
                    f,
                    "\t{} {}r[{}]({}), {}",
                    bop, r1.regsize, r1.vr, r1.rr, rhs
                )
            }
            Call(r, lb, args, usedrs) => {
                write!(
                    f,
                    "\t{}r[{}]({}) <- call ${} (arg * {}), (used register * {})",
                    r.regsize,
                    r.vr,
                    r.rr,
                    lb,
                    args.len(),
                    usedrs.len()
                )
            }
            Comp(op, dst, src, rorn) => match rorn {
                RegorNum::Reg(r) => {
                    if *op == CompOp::Ceqw {
                        write!(
                            f,
                            "\t{}r[{}]({}) <- {}r[{}]({}) == {}r[{}]({})",
                            dst.regsize,
                            dst.vr,
                            dst.rr,
                            src.regsize,
                            src.vr,
                            src.rr,
                            r.regsize,
                            r.vr,
                            r.rr
                        )
                    } else {
                        assert_eq!(*op, CompOp::Csltw);
                        write!(
                            f,
                            "\t{}r[{}]({}) <- {}r[{}]({}) < {}r[{}]({})",
                            dst.regsize,
                            dst.vr,
                            dst.rr,
                            src.regsize,
                            src.vr,
                            src.rr,
                            r.regsize,
                            r.vr,
                            r.rr
                        )
                    }
                }
                RegorNum::Num(num) => {
                    write!(
                        f,
                        "\t{}r[{}]({}) <- {}r[{}]({}) == {}",
                        dst.regsize, dst.vr, dst.rr, src.regsize, src.vr, src.rr, num
                    )
                }
            },
            Jnz(src, lb1, lb2) => {
                write!(
                    f,
                    "\t{}r[{}]({})? go {}: {}",
                    src.regsize, src.vr, src.rr, lb1, lb2
                )
            }
            Jmp(lb) => {
                write!(f, "\tjmp {}", lb)
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct LowIrBlock {
    pub lb: &'static str,
    pub instrs: Vec<LowIrInstr>,
}

impl LowIrBlock {
    pub fn new(lb: &'static str) -> Self {
        Self { lb, instrs: vec![] }
    }
    pub fn pushinstr(&mut self, rinstr: LowIrInstr, day: &mut i32) {
        *day += 1;
        self.instrs.push(rinstr)
    }
}

#[derive(Debug)]
pub struct LowIrFunction {
    pub lb: &'static str,
    pub rbbs: Vec<LowIrBlock>,
    pub framesize: i32,
}

impl LowIrFunction {
    pub fn new(lb: &'static str) -> Self {
        Self {
            lb,
            rbbs: vec![],
            framesize: -100,
        }
    }
    fn pushblock(&mut self, rbb: LowIrBlock) {
        self.rbbs.push(rbb)
    }
}

#[derive(Debug)]
pub struct LowIrProgram {
    pub funcs: Vec<LowIrFunction>,
}

impl LowIrProgram {
    pub fn new() -> Self {
        Self { funcs: vec![] }
    }
    pub fn pushfunc(&mut self, rgfun: LowIrFunction) {
        self.funcs.push(rgfun);
    }
}

fn evalparserinstr(
    pinstr: SsaInstr,
    rglf: &mut HashMap<i32, (i32, i32)>,
    vstkd: &mut HashMap<i32, i32>,
    rbb: &mut LowIrBlock,
    day: &mut i32,
    stackpointer: &mut i32,
) -> Option<Register> {
    use SsaInstrOp::*;
    match pinstr.op {
        Ret(fco) => match fco {
            FirstClassObj::Variable(var) => {
                let mut src = Register::new(var.frsn);
                src.btday = *day + 1;
                src.daday = *day + 1;
                if let Some((btday, _)) = rglf.get(&var.frsn) {
                    src.btday = *btday;
                }
                rbb.pushinstr(LowIrInstr::Ret(src), day);
                rglf.insert(src.vr, (src.btday, src.daday));
                Some(src)
            }
            FirstClassObj::Num(valty, num) => {
                let mut src = Register::new(nextfreshregister());
                src.btday = *day + 1;
                src.daday = *day + 1;
                rbb.pushinstr(LowIrInstr::Movenum(src, num), day);
                rglf.insert(src.vr, (src.btday, src.daday));
                rbb.pushinstr(LowIrInstr::Ret(src), day);
                Some(src)
            }
        },
        Assign(_valuety, var, pinstr) => {
            let mut src = evalparserinstr(
                *pinstr,
                rglf,
                vstkd,
                rbb,
                day,
                stackpointer,
            )
            .unwrap_or_else(|| panic!("evalparserinstr error: Assign"));
            let mut dst = Register::newall(var.frsn, *day + 1, *day + 1, var.ty.toregrefsize());
            if let Some((btday, _)) = rglf.get(&var.frsn) {
                dst.btday = *btday;
            }
            src.daday = *day + 1;
            rbb.pushinstr(LowIrInstr::Movereg(dst, src), day);
            rglf.insert(src.vr, (src.btday, src.daday));
            rglf.insert(dst.vr, (dst.btday, dst.daday));
            None
        }
        Alloc4(var, _align) => {
            *stackpointer += _align;
            let dst = Register::newall(var.frsn, *day, *day, var.ty.toregrefsize());
            assert_eq!(dst.regsize, 8);
            rglf.insert(dst.vr, (*day, *day));
            vstkd.insert(var.frsn, *stackpointer);
            None
        }
        Storew(fco, dstvar) => {
            let _bytesize = dstvar.ty.stacksize();
            // variable stack pointer
            let varsp = vstkd
                .get(&dstvar.frsn)
                .unwrap_or_else(|| panic!("var can't be found in Storew"));
            match fco {
                FirstClassObj::Num(valty, num) => {
                    rbb.pushinstr(LowIrInstr::Storewnum(num, *varsp), day);
                }
                FirstClassObj::Variable(srcvar) => {
                    let src;
                    if let Some((btday, _)) = rglf.get(&srcvar.frsn) {
                        src = Register::newall(
                            srcvar.frsn,
                            *btday,
                            *day + 1,
                            srcvar.ty.toregrefsize(),
                        );
                        rglf.insert(srcvar.frsn, (src.btday, src.daday));
                    } else {
                        panic!("{:?} is not defined", srcvar);
                    }
                    rbb.pushinstr(LowIrInstr::Storewreg(src, *varsp), day);
                }
            }
            None
        }
        Loadw(var) => {
            let (scbt, _) = rglf
                .get(&var.frsn)
                .unwrap_or_else(|| panic!("{:?} is not defined.", var));
            rglf.insert(var.frsn, (*scbt, *day + 1));
            let varsp = vstkd
                .get(&var.frsn)
                .unwrap_or_else(|| panic!("{:?} is not defined.", var));
            let src = Register::newall(nextfreshregister(), *day + 1, *day + 1, 4);
            rglf.insert(src.vr, (src.btday, src.daday));
            rbb.pushinstr(LowIrInstr::Loadw(src, *varsp), day);
            Some(src)
        }
        Bop(binop, lfco, rfco) => {
            let dst;
            if let FirstClassObj::Variable(v1) = lfco {
                let (v1birth, _) = rglf
                    .get(&v1.frsn)
                    .unwrap_or_else(|| panic!("{:?} is not defined.", v1));
                dst = Register::newall(v1.frsn, *v1birth, *day + 1, v1.ty.toregrefsize());
                rglf.insert(v1.frsn, (dst.btday, dst.daday));
            } else {
                panic!("Bop lhs error in lowir.{:?}", lfco);
            }
            match rfco {
                FirstClassObj::Variable(v2) => {
                    let (v2birth, _) = rglf
                        .get(&v2.frsn)
                        .unwrap_or_else(|| panic!("{:?} is not defined.", v2));
                    let src =
                        Register::newall(v2.frsn, *v2birth, *day + 1, v2.ty.toregrefsize());
                    rglf.insert(v2.frsn, (src.btday, src.daday));
                    rbb.pushinstr(LowIrInstr::Bop(binop, dst, RegorNum::Reg(src)), day);
                }
                FirstClassObj::Num(valty, num) => {
                    rbb.pushinstr(LowIrInstr::Bop(binop, dst, RegorNum::Num(num)), day);
                }
            }
            Some(dst)
        }
        Call(retty, funlb, args, variadic) => {
            let dst = Register::newall(
                nextfreshregister(),
                *day + 1,
                *day + 1,
                retty.toregrefsize(),
            );
            rglf.insert(dst.vr, (dst.btday, dst.daday));
            let mut newargs = vec![];
            for arg in args {
                newargs.push(fco2reg(arg, rglf, *day));
            }
            rbb.pushinstr(LowIrInstr::Call(dst, funlb, newargs, vec![]), day);
            Some(dst)
        }
        Comp(cop, dstv, srcv, fco) => {
            let dst = Register::newall(dstv.frsn, *day + 1, *day + 1, dstv.ty.toregrefsize());
            rglf.insert(dst.vr, (dst.btday, dst.daday));
            let (scbt, _) = rglf
                .get(&srcv.frsn)
                .unwrap_or_else(|| panic!("{:?} is not defined in Ceqw.", srcv));
            let src = Register::newall(srcv.frsn, *scbt, *day + 1, srcv.ty.toregrefsize());
            rglf.insert(srcv.frsn, (src.btday, src.daday));
            let rorn = fco2reg(fco, rglf, *day);
            rbb.pushinstr(LowIrInstr::Comp(cop, dst, src, rorn), day);
            None
        }
        Jnz(srcv, lb1, lb2) => {
            let (scbt, _) = rglf
                .get(&srcv.frsn)
                .unwrap_or_else(|| panic!("{:?} is not defined in Ceqw.", srcv));
            let src = Register::newall(srcv.frsn, *scbt, *day + 1, srcv.ty.toregrefsize());
            rglf.insert(srcv.frsn, (src.btday, src.daday));
            rbb.pushinstr(LowIrInstr::Jnz(src, lb1, lb2), day);
            None
        }
        Jmp(lb) => {
            rbb.pushinstr(LowIrInstr::Jmp(lb), day);
            None
        }
        Phi(_) => None,
    }
}

fn fco2reg(
    fco: FirstClassObj,
    rglf: &mut HashMap<i32, (i32, i32)>,
    day: i32,
) -> RegorNum {
    match fco {
        FirstClassObj::Variable(var) => {
            if let Some((btday, _)) = rglf.get(&var.frsn) {
                let r = Register::newall(var.frsn, *btday, day + 1, var.ty.toregrefsize());
                rglf.insert(var.frsn, (r.btday, r.daday));
                RegorNum::Reg(r)
            } else {
                panic!("{:?} is not defined", var);
            }
        }
        FirstClassObj::Num(valty, num) => RegorNum::Num(num),
    }
}

fn decidereglife(r: &mut Register, rglf: &mut HashMap<i32, (i32, i32)>) {
    if r.vr < 0 {
        return;
    }
    let (btday, daday) = rglf
        .get(&r.vr)
        .unwrap_or_else(|| panic!("Isn't it possible to come here? {:?}", rglf));
    (*r).btday = *btday;
    (*r).daday = *daday;
}

fn registerlifeupdate(lpg: &mut LowIrProgram, rglf: &mut HashMap<i32, (i32, i32)>) {
    for rfun in &mut lpg.funcs {
        for rbb in &mut rfun.rbbs {
            for rinstr in &mut rbb.instrs {
                use LowIrInstr::*;
                match rinstr {
                    Movenum(ref mut r, _)
                    | Storewreg(ref mut r, _)
                    | Ret(ref mut r)
                    | Loadw(ref mut r, _)
                    | Jnz(ref mut r, ..) => {
                        decidereglife(r, rglf);
                    }
                    Movereg(.., ref mut r1, ref mut r2) | Comp(_, ref mut r1, ref mut r2, _) => {
                        decidereglife(r1, rglf);
                        decidereglife(r2, rglf);
                    }
                    Call(ref mut r, _, ref mut args, _) => {
                        decidereglife(r, rglf);
                        for arg in args {
                            if let RegorNum::Reg(r) = arg {
                                decidereglife(r, rglf);
                            }
                        }
                    }
                    Bop(_, ref mut r1, ref mut r2) => {
                        decidereglife(r1, rglf);
                        if let RegorNum::Reg(ref mut r) = r2 {
                            decidereglife(r, rglf);
                        }
                    }
                    Storewnum(..) | Jmp(..) => {}
                }
            }
        }
    }
}

fn processfunarguments(args: &Vec<Var>, rglf: &mut HashMap<i32, (i32, i32)>) {
    for i in 0..args.len() {
        let r = Register::newall(-(i as i32 + 1), 0, std::i32::MAX, args[i].ty.toregrefsize());
        rglf.insert(r.vr, (r.btday, r.daday));
    }
}

pub fn genlowir(ppg: SsaProgram) -> LowIrProgram {
    let mut lpg = LowIrProgram::new();
    let mut day = 0;
    // manage register lifespan
    let mut rglf: HashMap<i32, (i32, i32)> = HashMap::new();
    // manage stackpointer of variables
    let mut vstkd = HashMap::new();
    // manage variable frsn and stackpointer
    for pfun in ppg.funcs {
        let mut rfun = LowIrFunction::new(pfun.name);
        let mut stackpointer = 0;
        // function arguments
        processfunarguments(&pfun.args, &mut rglf);
        for pbb in pfun.bls {
            let mut rbb = LowIrBlock::new(pbb.lb);
            for instr in pbb.instrs {
                if !instr.living {
                    continue;
                }
                evalparserinstr(
                    instr,
                    &mut rglf,
                    &mut vstkd,
                    &mut rbb,
                    &mut day,
                    &mut stackpointer,
                );
            }
            rfun.pushblock(rbb)
        }
        rfun.framesize = stackpointer;
        lpg.pushfunc(rfun);
    }
    registerlifeupdate(&mut lpg, &mut rglf);
    lpg
}
