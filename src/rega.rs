use super::lowir::*;

pub const GENEREGSIZE: usize = 7;

fn regaoflir(lir: &mut LowIrInstr, day: &mut i32, realregs: &mut [i32;GENEREGSIZE]) {
    use LowIrInstr::*;
    match lir {
        Movenum(ref mut r, _) | Ret(ref mut r) | Storewreg(ref mut r, _)
        | Loadw(ref mut r, _) => {    
            r.regalloc(realregs);
            if r.deathday == *day {
                realregs[r.rr as usize] = -1;
            }
        }
        Movereg(ref mut r1, ref mut r2) | Add(ref mut r1, ref mut r2) => {
            r1.regalloc(realregs);
            r2.regalloc(realregs);
            if r1.deathday == *day {
                realregs[r1.rr as usize] = -1;
            }
            if r2.deathday == *day {
                realregs[r2.rr as usize] = -1;
            }
        }
        Storewnum(..) => {}
    }
}

pub fn registeralloc(mut lpg: LowIrProgram) -> LowIrProgram {
    let mut day = 1;
    let mut realregs: [i32;GENEREGSIZE] = [-1, -1, -1, -1, -1, -1, -1];
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