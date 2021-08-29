use super::lowir::*;

pub const GENEREGSIZE: usize = 7;

fn regaoflir(lir: &mut LowIrInstr, day: &mut i32, realregs: &mut [i32; GENEREGSIZE]) {
    use LowIrInstr::*;
    match lir {
        Movenum(ref mut r, _)
        | Ret(ref mut r)
        | Storewreg(ref mut r, _)
        | Loadw(ref mut r, _)
        | Jnz(ref mut r, ..) => {
            r.regalloc(realregs);
            if r.daday == *day && r.vr >= 0 {
                realregs[r.rr as usize] = -1;
            }
        }
        Movereg(ref mut r1, ref mut r2) => {
            r1.regalloc(realregs);
            r2.regalloc(realregs);
            if r1.daday == *day && r1.vr >= 0 {
                realregs[r1.rr as usize] = -1;
            }
            if r2.daday == *day && r2.vr >= 0 {
                realregs[r2.rr as usize] = -1;
            }
        }
        Bop(_, ref mut r1, ref mut r2) => {
            r1.regalloc(realregs);
            if r1.daday == *day && r1.vr >= 0 {
                realregs[r1.rr as usize] = -1;
            }
            if let RegorNum::Reg(ref mut r) = r2 {
                r.regalloc(realregs);
                if r.daday == *day && r.vr >= 0 {
                    realregs[r.rr as usize] = -1;
                }
            }
        }
        Call(ref mut r, _, ref mut args, ref mut usedrs) => {
            for i in 0..realregs.len() {
                if realregs[i] != -1 {
                    usedrs.push(i);
                }
            }
            r.regalloc(realregs);
            let mut regargs = vec![];
            for ref mut arg in args {
                if let RegorNum::Reg(r2) = arg {
                    r2.regalloc(realregs);
                    regargs.push(*r2);
                }
            }
            if r.daday == *day && r.vr >= 0 {
                realregs[r.rr as usize] = -1;
            }
            for r2 in regargs {
                if r2.daday == *day {
                    realregs[r2.rr as usize] = -1;
                }
            }
        }
        Comp(_op, ref mut r1, ref mut r2, ref mut rorn) => {
            r1.regalloc(realregs);
            r2.regalloc(realregs);
            if let RegorNum::Reg(r3) = rorn {
                r3.regalloc(realregs);
                if r3.daday == *day && r3.vr >= 0 {
                    realregs[r2.rr as usize] = -1;
                }
            }
            if r1.daday == *day && r1.vr >= 0 {
                realregs[r1.rr as usize] = -1;
            }
            if r2.daday == *day && r2.vr >= 0 {
                realregs[r2.rr as usize] = -1;
            }
        }
        Storewnum(..) | Jmp(..) => {}
    }
}

pub fn registeralloc(mut lpg: LowIrProgram) -> LowIrProgram {
    let mut day = 1;
    let mut realregs: [i32; GENEREGSIZE] = [-1, -1, -1, -1, -1, -1, -1];
    for lowfunc in &mut lpg.funcs {
        for lowbb in &mut lowfunc.rbbs {
            for lowir in &mut lowbb.instrs {
                regaoflir(lowir, &mut day, &mut realregs);
                day += 1;
            }
        }
    }
    lpg
}
