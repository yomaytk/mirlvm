use super::dominators::*;
use super::parser::{SsaInstr, SsaInstrOp, SsaProgram};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemToregType {
    OneStore, // store only once
    OneBlock, // store and load in one basic block
    General,  // other than above
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemToregAlloca {
    pub name: &'static str,
    pub defbbs: HashSet<usize>,
    pub usgbbs: HashSet<usize>,
    pub strcnt: usize,
    pub ty: Option<MemToregType>,
}

impl MemToregAlloca {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            defbbs: HashSet::new(),
            usgbbs: HashSet::new(),
            strcnt: 0,
            ty: None,
        }
    }
    pub fn strpush(&mut self, bbid: usize) {
        self.defbbs.insert(bbid);
        self.strcnt += 1;
    }
    pub fn decision_type(mtamass: &mut HashMap<&'static str, Self>) {
        for (_, m2ralloc) in mtamass.iter_mut() {
            if m2ralloc.strcnt == 1 {
                m2ralloc.ty = Some(MemToregType::OneStore);
                continue;
            }
            if m2ralloc.defbbs.is_subset(&m2ralloc.usgbbs)
                && m2ralloc.usgbbs.is_subset(&m2ralloc.defbbs)
                && m2ralloc.defbbs.len() == 1
            {
                m2ralloc.ty = Some(MemToregType::OneBlock);
                continue;
            }
            m2ralloc.ty = Some(MemToregType::General);
        }
    }
    pub fn eztype(mtamass: &HashMap<&'static str, Self>, vne: &'static str) -> bool {
        use MemToregType::*;
        if let OneStore | OneBlock = mtamass
            .get(vne)
            .unwrap_or_else(|| panic!("{} don't be defined.", vne))
            .ty
            .unwrap_or_else(|| panic!("{}'s memtoregtype is not defined.", vne))
        {
            true
        } else {
            false
        }
    }
}

// process for OneStore or OneBlock MemToregType
pub fn ezmem2reg(spg: &mut SsaProgram) {
    for func in &mut spg.funcs {
        let m2rinfo = &func.m2rinfo;
        // hashmap for the latest src data for alloca var.
        let mut sthash: HashMap<&'static str, SsaInstrOp> = HashMap::new();
        for bb in &mut func.bls {
            for instr in &mut bb.instrs {
                use SsaInstrOp::*;
                match &instr.op {
                    Assign(vty, v, rhs) if matches!(&rhs.op, Loadw(_)) => {
                        let vne = rhs.getld_vn();
                        if MemToregAlloca::eztype(m2rinfo, vne) {
                            instr.op = Assign(
                                *vty,
                                v.clone(),
                                Box::new(SsaInstr::new(sthash.get(vne).unwrap().clone())),
                            );
                        }
                    }
                    Storew(fco, var) => {
                        if MemToregAlloca::eztype(m2rinfo, var.name) {
                            sthash.insert(&var.name, SsaInstrOp::Src(fco.clone()));
                            instr.op = Nop;
                        }
                    }
                    Alloc4(var, _) => {
                        if MemToregAlloca::eztype(m2rinfo, var.name) {
                            instr.op = Nop;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
