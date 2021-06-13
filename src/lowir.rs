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
    pub birthday: i32,
    pub deathday: i32,
    pub regsize: i32,
}

impl Register {
    pub fn new(vr: i32) -> Self {
        Self {
            vr,
            rr: NULLNUMBER,
            birthday: NULLNUMBER,
            deathday: NULLNUMBER,
            regsize: REGDEFASIZE,
        }
    }
    pub fn newall(vr: i32, birthday: i32, deathday: i32, regsize: i32) -> Self {
        Self {
            vr,
            rr: NULLNUMBER,
            birthday,
            deathday,
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
    Add(Register, Register),
    Call(Register, Label, Vec<RegorNum>, Vec<usize>),
    Ceqw(Register, Register, RegorNum),
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
            Add(r1, r2) => {
                write!(
                    f,
                    "\tadd {}r[{}]({}), {}r[{}]({})",
                    r1.regsize, r1.vr, r1.rr, r2.regsize, r2.vr, r2.rr
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
            Ceqw(dst, src, rorn) => {
                match rorn {
                    RegorNum::Reg(r) => {
                        write!(f, "\t{}r[{}]({}) <- {}r[{}]({}) == {}r[{}]({})", dst.regsize, dst.vr, dst.rr, src.regsize, src.vr, src.rr, r.regsize, r.vr, r.rr)
                    }
                    RegorNum::Num(num) => {
                        write!(f, "\t{}r[{}]({}) <- {}r[{}]({}) == {}", dst.regsize, dst.vr, dst.rr, src.regsize, src.vr, src.rr, num)
                    }
                }
            }
            Jnz(src, lb1, lb2) => {
                write!(f, "\t{}r[{}]({})? go {}: {}", src.regsize, src.vr, src.rr, lb1, lb2)
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
    register_lifedata: &mut HashMap<i32, (i32, i32)>,
    varstackdata: &mut HashMap<i32, i32>,
    rbb: &mut LowIrBlock,
    day: &mut i32,
    stackpointer: &mut i32,
) -> Option<Register> {
    use SsaInstr::*;
    match pinstr {
        Ret(fco) => match fco {
            FirstClassObj::Variable(var) => {
                let mut src = Register::new(var.freshnum);
                src.birthday = *day + 1;
                src.deathday = *day + 1;
                if let Some((birthday, _)) = register_lifedata.get(&var.freshnum) {
                    src.birthday = *birthday;
                }
                rbb.pushinstr(LowIrInstr::Ret(src), day);
                register_lifedata.insert(src.vr, (src.birthday, src.deathday));
                Some(src)
            }
            FirstClassObj::Num(num) => {
                let mut src = Register::new(nextfreshregister());
                src.birthday = *day + 1;
                src.deathday = *day + 1;
                rbb.pushinstr(LowIrInstr::Movenum(src, num), day);
                register_lifedata.insert(src.vr, (src.birthday, src.deathday));
                Some(src)
            }
        },
        Assign(valuety, var, pinstr) => {
            let mut src = evalparserinstr(
                *pinstr,
                register_lifedata,
                varstackdata,
                rbb,
                day,
                stackpointer,
            )
            .unwrap_or_else(|| panic!("evalparserinstr error: Assign"));
            let mut dst = Register::newall(var.freshnum, *day + 1, *day + 1, var.ty.toregrefsize());
            if let Some((birthday, _)) = register_lifedata.get(&var.freshnum) {
                dst.birthday = *birthday;
            }
            src.deathday = *day + 1;
            rbb.pushinstr(LowIrInstr::Movereg(dst, src), day);
            register_lifedata.insert(src.vr, (src.birthday, src.deathday));
            register_lifedata.insert(dst.vr, (dst.birthday, dst.deathday));
            None
        }
        Alloc4(var, _align) => {
            *stackpointer += _align;
            let dst = Register::newall(var.freshnum, *day, *day, var.ty.toregrefsize());
            assert_eq!(dst.regsize, 8);
            register_lifedata.insert(dst.vr, (*day, *day));
            varstackdata.insert(var.freshnum, *stackpointer);
            None
        }
        Storew(fco, dstvar) => {
            let bytesize = dstvar.ty.stacksize();
            // variable stack pointer
            let varsp = varstackdata
                .get(&dstvar.freshnum)
                .unwrap_or_else(|| panic!("var can't be found in Storew"));
            match fco {
                FirstClassObj::Num(num) => {
                    rbb.pushinstr(LowIrInstr::Storewnum(num, *varsp), day);
                }
                FirstClassObj::Variable(srcvar) => {
                    let src;
                    if let Some((birthday, _)) = register_lifedata.get(&srcvar.freshnum) {
                        src = Register::newall(
                            srcvar.freshnum,
                            *birthday,
                            *day + 1,
                            srcvar.ty.toregrefsize(),
                        );
                        register_lifedata.insert(srcvar.freshnum, (src.birthday, src.deathday));
                    } else {
                        panic!("{:?} is not defined", srcvar);
                    }
                    rbb.pushinstr(LowIrInstr::Storewreg(src, *varsp), day);
                }
            }
            None
        }
        Loadw(var) => {
            let (srcbirth, _) = register_lifedata
                .get(&var.freshnum)
                .unwrap_or_else(|| panic!("{:?} is not defined.", var));
            register_lifedata.insert(var.freshnum, (*srcbirth, *day + 1));
            let varsp = varstackdata
                .get(&var.freshnum)
                .unwrap_or_else(|| panic!("{:?} is not defined.", var));
            let src = Register::newall(nextfreshregister(), *day + 1, *day + 1, 4);
            register_lifedata.insert(src.vr, (src.birthday, src.deathday));
            rbb.pushinstr(LowIrInstr::Loadw(src, *varsp), day);
            Some(src)
        }
        Add(lfco, rfco) => {
            // TODO (Assume lfco and rfco are register)
            if let (FirstClassObj::Variable(v1), FirstClassObj::Variable(v2)) = (lfco, rfco) {
                let (v1birth, _) = register_lifedata
                    .get(&v1.freshnum)
                    .unwrap_or_else(|| panic!("{:?} is not defined.", v1));
                let (v2birth, _) = register_lifedata
                    .get(&v2.freshnum)
                    .unwrap_or_else(|| panic!("{:?} is not defined.", v2));
                let dst = Register::newall(v1.freshnum, *v1birth, *day + 1, v1.ty.toregrefsize());
                let src = Register::newall(v2.freshnum, *v2birth, *day + 1, v2.ty.toregrefsize());
                register_lifedata.insert(v1.freshnum, (dst.birthday, dst.deathday));
                register_lifedata.insert(v2.freshnum, (src.birthday, src.deathday));
                rbb.pushinstr(LowIrInstr::Add(dst, src), day);
                return Some(dst)
            }
            panic!("Don't come here at your current level")
        }
        Call(retty, funlb, args) => {
            let dst = Register::newall(
                nextfreshregister(),
                *day + 1,
                *day + 1,
                retty.toregrefsize(),
            );
            register_lifedata.insert(dst.vr, (dst.birthday, dst.deathday));
            let mut newargs = vec![];
            for arg in args {
                newargs.push(fco2reg(arg, register_lifedata, *day));
            }
            rbb.pushinstr(LowIrInstr::Call(dst, funlb, newargs, vec![]), day);
            Some(dst)
        }
        Ceqw(dstv, srcv, fco) => {
            let dst = Register::newall(dstv.freshnum, *day+1, *day+1, dstv.ty.toregrefsize());
            register_lifedata.insert(dst.vr, (dst.birthday, dst.deathday));
            let (srcbirth, _) = register_lifedata.get(&srcv.freshnum).unwrap_or_else(|| panic!("{:?} is not defined in Ceqw.", srcv));
            let src = Register::newall(srcv.freshnum, *srcbirth, *day+1, srcv.ty.toregrefsize());
            register_lifedata.insert(srcv.freshnum, (src.birthday, src.deathday));
            let rorn = fco2reg(fco, register_lifedata, *day);
            rbb.pushinstr(LowIrInstr::Ceqw(dst, src, rorn), day);
            None
        }
        Jnz(srcv, lb1, lb2) => {
            let (srcbirth, _) = register_lifedata.get(&srcv.freshnum).unwrap_or_else(|| panic!("{:?} is not defined in Ceqw.", srcv));
            let src = Register::newall(srcv.freshnum, *srcbirth, *day+1, srcv.ty.toregrefsize());
            register_lifedata.insert(srcv.freshnum, (src.birthday, src.deathday));
            rbb.pushinstr(LowIrInstr::Jnz(src, lb1, lb2), day);
            None
        }
        Jmp(lb) => {
            rbb.pushinstr(LowIrInstr::Jmp(lb), day);
            None
        }
    }
}

fn fco2reg(fco: FirstClassObj, register_lifedata: &mut HashMap<i32, (i32, i32)>, day: i32) -> RegorNum {
    match fco {
        FirstClassObj::Variable(var) => {
            if let Some((birthday, _)) = register_lifedata.get(&var.freshnum) {
                let r = Register::newall(
                    var.freshnum,
                    *birthday,
                    day + 1,
                    var.ty.toregrefsize(),
                );
                register_lifedata.insert(var.freshnum, (r.birthday, r.deathday));
                RegorNum::Reg(r)
            } else {
                panic!("{:?} is not defined", var);
            }
        }
        FirstClassObj::Num(num) => {
            RegorNum::Num(num)
        }
    }
}

fn decidereglife(r: &mut Register, register_lifedata: &mut HashMap<i32, (i32, i32)>) {
    if r.vr < 0 {
        return;
    }
    let (birthday, deathday) = register_lifedata
        .get(&r.vr)
        .unwrap_or_else(|| panic!("Isn't it possible to come here? {:?}", register_lifedata));
    (*r).birthday = *birthday;
    (*r).deathday = *deathday;
}

fn registerlifeupdate(lpg: &mut LowIrProgram, register_lifedata: &mut HashMap<i32, (i32, i32)>) {
    for rfun in &mut lpg.funcs {
        for rbb in &mut rfun.rbbs {
            for rinstr in &mut rbb.instrs {
                use LowIrInstr::*;
                match rinstr {
                    Movenum(ref mut r, _) | Storewreg(ref mut r, _) | Ret(ref mut r) | Loadw(ref mut r, _) | Jnz(ref mut r, ..) => {
                        decidereglife(r, register_lifedata);
                    }
                    Movereg(.., ref mut r1, ref mut r2) | Add(ref mut r1, ref mut r2) | Ceqw(ref mut r1, ref mut r2, _) => {
                        decidereglife(r1, register_lifedata);
                        decidereglife(r2, register_lifedata);
                    }
                    Call(ref mut r, _, ref mut args, _) => {
                        decidereglife(r, register_lifedata);
                        for arg in args {
                            if let RegorNum::Reg(r) = arg {
                                decidereglife(r, register_lifedata);
                            }
                        }
                    }
                    Storewnum(..) | Jmp(..) => {}
                }
            }
        }
    }
}

fn processfunarguments(args: &Vec<Var>, register_lifedata: &mut HashMap<i32, (i32, i32)>) {
    for i in 0..args.len() {
        let r = Register::newall(-(i as i32 + 1), 0, std::i32::MAX, args[i].ty.toregrefsize());
        register_lifedata.insert(r.vr, (r.birthday, r.deathday));
    }
}

pub fn genlowir(ppg: SsaProgram) -> LowIrProgram {
    let mut lpg = LowIrProgram::new();
    let mut day = 0;
    // manage register lifespan
    let mut register_lifedata: HashMap<i32, (i32, i32)> = HashMap::new();
    // manage stackpointer of variables
    let mut varstackdata = HashMap::new();
    // manage variable freshnum and stackpointer
    for pfun in ppg.funcs {
        let mut rfun = LowIrFunction::new(pfun.name);
        let mut stackpointer = 0;
        // function arguments
        processfunarguments(&pfun.args, &mut register_lifedata);
        // panic!("{:?}", register_lifedata);
        for pbb in pfun.bls {
            let mut rbb = LowIrBlock::new(pbb.lb);
            for instr in pbb.instrs {
                evalparserinstr(
                    instr,
                    &mut register_lifedata,
                    &mut varstackdata,
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
    registerlifeupdate(&mut lpg, &mut register_lifedata);
    lpg
}
