use super::*;
use super::parser::*;
use std::collections::HashMap;

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
    pub fn newwithlife(vr: i32, birthday: i32, deathday: i32) -> Self {
        Self {
            vr,
            rr: NULLNUMBER,
            birthday,
            deathday,
            regsize: REGDEFASIZE,
        }
    }
    pub fn newall(vr: i32, birthday: i32, deathday: i32, regsize: i32) -> Self {
        Self {
            vr,
            rr: NULLNUMBER,
            birthday,
            deathday,
            regsize
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RegInstr {
    Movenum(ValueType, Register, i32),
    Movereg(ValueType, Register, Register),
    Ret(Register),
    Storewreg(ByteSize, Register, i32),
    Storewnum(ByteSize, i32, i32),
    Loadw(Register, i32),
    Add(Register, Register),
}

#[derive(Clone, Debug)]
pub struct RegaBlock {
    pub lb: Label,
    pub instrs: Vec<RegInstr>,
}

impl RegaBlock {
    pub fn new(lb: Label) -> Self {
        Self {
            lb,
            instrs: vec![],
        }
    }
    pub fn pushinstr(&mut self, rinstr: RegInstr, day: &mut i32) {
        *day += 1;
        self.instrs.push(rinstr)
    }
}

#[derive(Debug)]
pub struct RegaFunction {
    pub lb: Label,
    pub rbbs: Vec<RegaBlock>,
    pub framesize: i32
}

impl RegaFunction {
    pub fn new(lb: Label) -> Self {
        Self {
            lb,
            rbbs: vec![],
            framesize: -100,
        }
    }
    fn pushblock(&mut self, rbb: RegaBlock) {
        self.rbbs.push(rbb)
    }
}

#[derive(Debug)]
pub struct RegaProgram {
    pub funcs: Vec<RegaFunction>
}

impl RegaProgram {
    pub fn new() -> Self {
        Self {
            funcs: vec![]
        }
    }
    pub fn pushfunc(&mut self, rgfun: RegaFunction) {
        self.funcs.push(rgfun);
    }
}

fn evalparserinstr(pinstr: ParserInstr, register_lifedata: &mut HashMap<i32, (i32, i32)>, varstackdata: &mut HashMap<i32, i32>,
    rbb: &mut RegaBlock, day: &mut i32, stackpointer: &mut i32) -> Option<Register> {
    use ParserInstr::*;
    match pinstr {
        Ret(fco) => {
            match fco {
                FirstClassObj::Variable(var) => {
                    let mut src = Register::new(var.freshnum);
                    src.birthday = *day + 1;
                    src.deathday = *day + 1;
                    if let Some((birthday, _)) = register_lifedata.get(&var.freshnum) {
                        src.birthday = *birthday;
                    }
                    rbb.pushinstr(RegInstr::Ret(src), day);
                    register_lifedata.insert(src.vr, (src.birthday, src.deathday));
                    Some(src)
                }
                FirstClassObj::Num(num) => {
                    let mut src = Register::new(nextfreshregister());
                    src.birthday = *day + 1;
                    src.deathday = *day + 1;
                    rbb.pushinstr(RegInstr::Movenum(ValueType::Word, src, num), day);
                    register_lifedata.insert(src.vr, (src.birthday, src.deathday));
                    Some(src)
                }
            }
        }
        Assign(valuety, var, pinstr) => {
            let src = evalparserinstr(*pinstr, register_lifedata, varstackdata, rbb, day, stackpointer).unwrap_or_else(|| panic!("evalparserinstr error: Assign"));
            let mut dst = Register::newall(var.freshnum, *day+1, *day+1, var.ty.toregrefsize());
            if let Some((birthday, _)) = register_lifedata.get(&var.freshnum) {
                dst.birthday = *birthday;
            }
            rbb.pushinstr(RegInstr::Movereg(valuety, dst, src), day);
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
            let varsp = varstackdata.get(&dstvar.freshnum).unwrap_or_else(|| panic!("var can't be found in Storew"));
            match fco {
                FirstClassObj::Num(num) => {
                    rbb.pushinstr(RegInstr::Storewnum(bytesize, num, *varsp), day);
                }
                FirstClassObj::Variable(srcvar) => {
                    if let Some((birthday, _)) = register_lifedata.get(&srcvar.freshnum) {
                        register_lifedata.insert(srcvar.freshnum, (*birthday, *day+1));
                    } else {
                        panic!("{:?} is not defined", srcvar);
                    }
                    let src = Register::new(srcvar.freshnum);
                    rbb.pushinstr(RegInstr::Storewreg(bytesize, src, *varsp), day);
                }
            }
            None
        }
        Loadw(var) => {
            let (srcbirth, _) = register_lifedata.get(&var.freshnum).unwrap_or_else(|| { panic!("{:?} is not defined.", var) });
            register_lifedata.insert(var.freshnum, (*srcbirth, *day+1));
            let varsp = varstackdata.get(&var.freshnum).unwrap_or_else(|| { panic!("{:?} is not defined.", var) });
            let src = Register::newall(nextfreshregister(), *day+1, *day+1, var.ty.toregrefsize());
            register_lifedata.insert(src.vr, (src.birthday, src.deathday));
            rbb.pushinstr(RegInstr::Loadw(src, *varsp), day);
            Some(src)
        }
        Add(lfco, rfco) => {
            // TODO (Assume lfco and rfco are register)
            if let (FirstClassObj::Variable(v1), FirstClassObj::Variable(v2)) = (lfco, rfco) {
                let (v1birth, _) = register_lifedata.get(&v1.freshnum).unwrap_or_else(|| { panic!("{:?} is not defined.", v1) });
                let (v2birth, _) = register_lifedata.get(&v2.freshnum).unwrap_or_else(|| { panic!("{:?} is not defined.", v2)});
                let dst = Register::newall(v1.freshnum, *v1birth, *day+1, v1.ty.toregrefsize());
                let src = Register::newall(v2.freshnum, *v2birth, *day+1, v2.ty.toregrefsize());
                register_lifedata.insert(v1.freshnum, (dst.birthday, dst.deathday));
                register_lifedata.insert(v2.freshnum, (src.birthday, src.deathday));
                rbb.pushinstr(RegInstr::Add(dst, src), day);
                return Some(dst)
            }
            panic!("Don't come here at your current level")
        }
    }
}

fn decidereglife(r: &mut Register, register_lifedata: &mut HashMap<i32, (i32, i32)>) {
    let (birthday, deathday) = register_lifedata.get(&r.vr).unwrap_or_else(|| { panic!("Isn't it possible to come here? {:?}", register_lifedata) });
    (*r).birthday = *birthday;
    (*r).deathday = *deathday;
}

fn registerlifeupdate(rgp: &mut RegaProgram, register_lifedata: &mut HashMap<i32, (i32, i32)>) {
    for rfun in &mut rgp.funcs {
        for rbb in &mut rfun.rbbs {
            for rinstr in &mut rbb.instrs {
                use RegInstr::*;
                match rinstr {
                    Movenum(_, ref mut r1, _) => { decidereglife(r1, register_lifedata); } 
                    Movereg(.., ref mut r1, ref mut r2) => { 
                        decidereglife(r1, register_lifedata);
                        decidereglife(r2, register_lifedata); 
                    }
                    Ret(r) => { decidereglife(r, register_lifedata); }
                    Storewreg(_, ref mut r, _) => { decidereglife(r, register_lifedata); }
                    Storewnum(..) => {}
                    Loadw(ref mut r, _) => { decidereglife(r, register_lifedata); }
                    Add(ref mut r1, ref mut r2) => {
                        decidereglife(r1, register_lifedata);
                        decidereglife(r2, register_lifedata);
                    }
                }
            }
        }
    }
}

pub fn genlowir(ppg: ParserProgram) -> RegaProgram {
    let mut rgp = RegaProgram::new();
    let mut day = 0;
    // manage register lifespan
    let mut register_lifedata: HashMap<i32, (i32, i32)> = HashMap::new();
    let mut varstackdata = HashMap::new();
    // manage variable freshnum and stackpointer
    for pfun in ppg.funcs {
        let mut rfun = RegaFunction::new(pfun.name.clone());
        let mut stackpointer = 0;
        // TODO function arguments
        for pbb in pfun.bls {
            let mut rbb = RegaBlock::new(pbb.lb.clone());
            for instr in pbb.instrs {
                evalparserinstr(instr, &mut register_lifedata, &mut varstackdata, &mut rbb, &mut day, &mut stackpointer);
            }
            rfun.pushblock(rbb)
        }
        rfun.framesize = stackpointer;
        rgp.pushfunc(rfun);
    }
    registerlifeupdate(&mut rgp, &mut register_lifedata);
    rgp
}