use super::parser::{
    nextfreshregister, FirstClassObj, SsaInstr, SsaInstrOp, SsaProgram, ValueType, Var, VarType,
};
use super::*;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

type VarLabel = &'static str;

pub static FRESH_TMP_V_MAP: Lazy<Mutex<HashMap<usize, &'static str>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
pub static FRESH_N_TMP: Lazy<Mutex<usize>> = Lazy::new(|| Mutex::new(0));

fn new_freshnum_tmp() -> usize {
    let res = *FRESH_N_TMP.lock().unwrap();
    *FRESH_N_TMP.lock().unwrap() = res + 1;
    res
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq)]
struct Life {
    bt: usize,
    dt: usize,
}

impl Life {
    fn new(bt: usize, dt: usize) -> Self {
        Self { bt, dt }
    }
}

trait UpLife {
    fn update(&mut self, varlb: VarLabel, day: usize);
}

impl UpLife for HashMap<VarLabel, Life> {
    fn update(&mut self, varlb: VarLabel, day: usize) {
        if self.contains_key(varlb) {
            self.get_mut(varlb).unwrap().dt = day;
        } else {
            self.insert(varlb, Life::new(day, day));
        }
    }
}

fn sub_cal_var_lifes(instr: &SsaInstr, lifes: &mut HashMap<VarLabel, Life>, day: usize) {
    use SsaInstrOp::*;
    match &instr.op {
        Ret(fco) | Src(fco) => {
            if let Some(varlb) = fco.get_varlb() {
                lifes.update(varlb, day);
            }
        }
        Assign(_, var, rhs) => {
            lifes.update(var.name, day);
            sub_cal_var_lifes(rhs, lifes, day);
        }
        Alloc4(var, _) | Loadw(var) | Jnz(var, ..) => {
            lifes.update(var.name, day);
        }
        Storew(fco, var) => {
            if let Some(varlb) = fco.get_varlb() {
                lifes.update(varlb, day);
            }
            lifes.update(var.name, day);
        }
        Bop(_, fco1, fco2) => {
            if let Some(varlb) = fco1.get_varlb() {
                lifes.update(varlb, day);
            }
            if let Some(varlb) = fco2.get_varlb() {
                lifes.update(varlb, day);
            }
        }
        Call(_, _, fcovs, _) => {
            for fco in fcovs {
                if let Some(varlb) = fco.get_varlb() {
                    lifes.update(varlb, day);
                }
            }
        }
        Comp(_, var1, var2, fco) => {
            lifes.update(var1.name, day);
            lifes.update(var2.name, day);
            if let Some(varlb) = fco.get_varlb() {
                lifes.update(varlb, day);
            }
        }
        Phi(_, vecs) => {
            for (_, fco) in vecs {
                if let Some(varlb) = fco.get_varlb() {
                    lifes.update(varlb, day);
                }
            }
        }
        Jmp(..) | Nop => {}
        DummyOp => panic!("cal_var_lifes DummyOp error."),
    }
}

fn cal_var_lifes(
    spg: &mut SsaProgram,
) -> (HashMap<VarLabel, Life>, HashMap<BBLabel, (usize, usize)>) {
    let mut day = 0;
    let mut lifes = HashMap::new();
    let mut bb_instr_lifes = HashMap::new();
    for func in &spg.funcs {
        for bb in &func.bls {
            bb_instr_lifes.insert(bb.lb, (day, usize::MAX));
            if bb.instrs.is_empty() {
                bb_instr_lifes.insert(bb.lb, (usize::MAX, usize::MAX));
                continue;
            }
            for instr in &bb.instrs {
                sub_cal_var_lifes(instr, &mut lifes, day);
                day += 1;
            }
            let st_day = bb_instr_lifes.get(bb.lb).unwrap().0;
            bb_instr_lifes.insert(bb.lb, (st_day, day - 1));
        }
    }
    (lifes, bb_instr_lifes)
}

pub fn rev_ssa(spg: &mut SsaProgram) {
    let (lifes, bb_instr_lifes) = cal_var_lifes(spg);
    let mut day = 0;
    use SsaInstrOp::*;
    for func in &mut spg.funcs {
        let mut bb_lbid_hash = HashMap::new();
        let mut proxy_instrs = vec![vec![]; func.bls.len()];
        for bb in &func.bls {
            bb_lbid_hash.insert(bb.lb, bb.id);
        }
        for bb in &mut func.bls {
            let instrs = std::mem::replace(&mut bb.instrs, vec![]);
            let mut tmp_var_copy_instrs = std::mem::replace(&mut proxy_instrs[bb.id], vec![]);
            for mut instr in instrs {
                match &instr.op {
                    Assign(vty, var, rhs) if matches!(&rhs.op, Phi(_, _)) => {
                        let var_life = lifes.get(var.name).unwrap();
                        let phi_vecs = rhs.op.get_phi_vec().unwrap();
                        let add_var;
                        // insert "tmp = x_i" to target basic block.
                        let mut need_tmp = false;
                        for (lb, _) in &phi_vecs {
                            let (st, ed) = *bb_instr_lifes.get(lb).unwrap();
                            need_tmp |= (var_life.bt >= st && var_life.bt <= ed)
                                || (var_life.dt >= st && var_life.dt <= ed);
                        }
                        // need tmp var
                        if need_tmp {
                            let freshn = new_freshnum_tmp();
                            let varlb = format!("tmp#_{}", freshn);
                            FRESH_TMP_V_MAP
                                .lock()
                                .unwrap()
                                .insert(freshn, Box::leak(varlb.into_boxed_str()));
                            add_var = Var::new(
                                FRESH_TMP_V_MAP.lock().unwrap().get(&freshn).unwrap(),
                                VarType::Word,
                                nextfreshregister(),
                            );
                            instr.op = Assign(
                                *vty,
                                var.clone(),
                                Box::new(SsaInstr::new_all(
                                    SsaInstrOp::Src(FirstClassObj::Variable(add_var.clone())),
                                    true,
                                    bb.lb,
                                )),
                            )
                        // simple method is sufficient.
                        } else {
                            add_var = var.clone();
                            instr = SsaInstr::new_all(Nop, true, bb.lb);
                        }
                        for (lb, fco) in phi_vecs {
                            let bb_id = bb_lbid_hash.get(lb).unwrap();
                            proxy_instrs[*bb_id].push(SsaInstr::new_all(
                                Assign(
                                    ValueType::Word,
                                    add_var.clone(),
                                    Box::new(SsaInstr::new_all(Src(fco.clone()), true, lb)),
                                ),
                                true,
                                lb,
                            ));
                        }
                    }
                    _ => {}
                }
                proxy_instrs[bb.id].push(instr);
                day += 1;
            }
            proxy_instrs[bb.id].append(&mut tmp_var_copy_instrs);
            bb.instrs = std::mem::replace(&mut proxy_instrs[bb.id], vec![]);
        }
        for bb in &mut func.bls {
            bb.instrs.append(&mut proxy_instrs[bb.id]);
        }
        proxy_instrs.clear();
    }
}
