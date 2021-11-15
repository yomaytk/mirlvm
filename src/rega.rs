use super::lowir::{
    LowIrBlock, LowIrInstr, LowIrProgram, NeedStack, Register, RegorNum, StashStacked,
};

pub const GENEREGSIZE: usize = 6;

fn regaoflir(
    lbb: &mut LowIrBlock,
    day: &mut i32,
    realregs: &mut [Option<Register>; GENEREGSIZE],
    ir_id: usize,
    stash_stacked: &mut StashStacked,
    var_frame_size: i32,
) -> Vec<LowIrInstr> {
    use LowIrInstr::*;
    let mut get_stash_reg_instrs = vec![];
    let mut target_instr = std::mem::replace(&mut lbb.instrs[ir_id], LowIrInstr::LowNop);
    match &mut target_instr {
        Movenum(ref mut r, _)
        | Ret(ref mut r)
        | Storewreg(ref mut r, _)
        | Loadw(ref mut r, _)
        | Jnz(ref mut r, ..) => {
            let needstack = r.regalloc(realregs, stash_stacked);
            get_stash_register(
                needstack,
                &mut get_stash_reg_instrs,
                r,
                realregs,
                var_frame_size,
            );
            if r.daday == *day && r.vr >= 0 {
                realregs[r.rr as usize] = None;
            }
        }
        Movereg(ref mut r1, ref mut r2) => {
            let needstack1 = r1.regalloc(realregs, stash_stacked);
            get_stash_register(
                needstack1,
                &mut get_stash_reg_instrs,
                r1,
                realregs,
                var_frame_size,
            );
            let needstack2 = r2.regalloc(realregs, stash_stacked);
            get_stash_register(
                needstack2,
                &mut get_stash_reg_instrs,
                r2,
                realregs,
                var_frame_size,
            );
            if r1.daday == *day && r1.vr >= 0 {
                realregs[r1.rr as usize] = None;
            }
            if r2.daday == *day && r2.vr >= 0 {
                realregs[r2.rr as usize] = None;
            }
        }
        Bop(_, ref mut r1, ref mut r2) => {
            let needstack1 = r1.regalloc(realregs, stash_stacked);
            get_stash_register(
                needstack1,
                &mut get_stash_reg_instrs,
                r1,
                realregs,
                var_frame_size,
            );
            if r1.daday == *day && r1.vr >= 0 {
                realregs[r1.rr as usize] = None;
            }
            if let RegorNum::Reg(ref mut r) = r2 {
                let needstack2 = r.regalloc(realregs, stash_stacked);
                get_stash_register(
                    needstack2,
                    &mut get_stash_reg_instrs,
                    r,
                    realregs,
                    var_frame_size,
                );
                if r.daday == *day && r.vr >= 0 {
                    realregs[r.rr as usize] = None;
                }
            }
        }
        Call(ref mut r, _, ref mut args, ref mut usedrs) => {
            for i in 0..realregs.len() {
                if realregs[i] != None {
                    usedrs.push(i);
                }
            }
            let needstack = r.regalloc(realregs, stash_stacked);
            get_stash_register(
                needstack,
                &mut get_stash_reg_instrs,
                r,
                realregs,
                var_frame_size,
            );
            let mut regargs = vec![];
            for ref mut arg in args {
                if let RegorNum::Reg(r2) = arg {
                    let needstack2 = r2.regalloc(realregs, stash_stacked);
                    get_stash_register(
                        needstack2,
                        &mut get_stash_reg_instrs,
                        r2,
                        realregs,
                        var_frame_size,
                    );
                    regargs.push(*r2);
                }
            }
            if r.daday == *day && r.vr >= 0 {
                realregs[r.rr as usize] = None;
            }
            for r2 in regargs {
                if r2.daday == *day {
                    if r2.rr > 6 {
                        panic!("{:?}", r2);
                    }
                    realregs[r2.rr as usize] = None;
                }
            }
        }
        Comp(_op, ref mut r1, ref mut r2, ref mut rorn) => {
            let needstack1 = r1.regalloc(realregs, stash_stacked);
            get_stash_register(
                needstack1,
                &mut get_stash_reg_instrs,
                r1,
                realregs,
                var_frame_size,
            );
            let needstack2 = r2.regalloc(realregs, stash_stacked);
            get_stash_register(
                needstack2,
                &mut get_stash_reg_instrs,
                r2,
                realregs,
                var_frame_size,
            );
            if let RegorNum::Reg(r3) = rorn {
                let needstack3 = r3.regalloc(realregs, stash_stacked);
                get_stash_register(
                    needstack3,
                    &mut get_stash_reg_instrs,
                    r3,
                    realregs,
                    var_frame_size,
                );
                if r3.daday == *day && r3.vr >= 0 {
                    realregs[r2.rr as usize] = None;
                }
            }
            if r1.daday == *day && r1.vr >= 0 {
                realregs[r1.rr as usize] = None;
            }
            if r2.daday == *day && r2.vr >= 0 {
                realregs[r2.rr as usize] = None;
            }
        }
        Storewnum(..) | Jmp(..) => {}
        LowNop => {
            panic!("impossible to reach LowNop instr.");
        }
    }
    get_stash_reg_instrs.push(target_instr);
    get_stash_reg_instrs
}

fn get_stash_register(
    needstack: NeedStack,
    get_stash_reg_instrs: &mut Vec<LowIrInstr>,
    reg: &Register,
    realregs: &[Option<Register>; GENEREGSIZE],
    var_frame_size: i32,
) {
    use NeedStack::*;
    match needstack {
        Exist(offset) => {
            let stack_offset = var_frame_size + offset;
            let mut tmp_reg = reg.clone();
            // use 7th register for buffer register
            tmp_reg.rr = 6;
            get_stash_reg_instrs.push(LowIrInstr::Loadw(tmp_reg.clone(), stack_offset));
            get_stash_reg_instrs.push(LowIrInstr::Storewreg(
                realregs[0].unwrap().clone(),
                stack_offset,
            ));
            get_stash_reg_instrs.push(LowIrInstr::Movereg(tmp_reg, realregs[0].unwrap().clone()));
        }
        NoExist(offset) => {
            let stack_offset = var_frame_size + offset;
            get_stash_reg_instrs.push(LowIrInstr::Storewreg(
                realregs[0].unwrap().clone(),
                stack_offset,
            ));
        }
        NoNeed => {}
    }
}

pub fn registeralloc(mut lpg: LowIrProgram) -> LowIrProgram {
    let mut day = 1;
    let mut realregs: [Option<Register>; GENEREGSIZE] = [None; GENEREGSIZE];
    for lowfunc in &mut lpg.funcs {
        let mut stash_stacked = StashStacked::new();
        for lowbb in &mut lowfunc.rbbs {
            let mut new_instrs = vec![];
            for ir_id in 0..lowbb.instrs.len() {
                let mut instrs = regaoflir(
                    lowbb,
                    &mut day,
                    &mut realregs,
                    ir_id,
                    &mut stash_stacked,
                    lowfunc.framesize,
                );
                day += 1;
                new_instrs.append(&mut instrs);
            }
            lowbb.instrs = new_instrs;
        }
        lowfunc.framesize += stash_stacked.vecs.len() as i32 * 8;
    }
    lpg
}
