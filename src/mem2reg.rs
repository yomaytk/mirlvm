use super::dominators::ControlFlowGraph;
use super::parser::{
    nextfreshregister, FirstClassObj, SsaBlock, SsaInstr, SsaInstrOp, SsaProgram, ValueType, Var,
    VarType,
};
use super::*;
use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

pub static FRESH_NUM_MAP: Lazy<Mutex<HashMap<usize, &'static str>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
pub static FRESH_N: Lazy<Mutex<usize>> = Lazy::new(|| Mutex::new(0));

fn new_freshnum_phi() -> usize {
    let res = *FRESH_N.lock().unwrap();
    *FRESH_N.lock().unwrap() = res + 1;
    res
}

trait OwnHash<K: std::cmp::Eq + std::hash::Hash, V: std::clone::Clone> {
    type K;
    type V;
    fn get_into(&mut self, key: K) -> Option<V>;
}

impl<K: std::cmp::Eq + std::hash::Hash, V: std::clone::Clone> OwnHash<K, V> for HashMap<K, V> {
    type K = Label;
    type V = (Label, Vec<(Label, FirstClassObj)>);
    fn get_into(&mut self, key: K) -> Option<V> {
        let value = self.get(&key);
        if let Some(_) = value {
            let res = Some(value.unwrap().clone());
            self.remove(&key);
            res
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemToregType {
    OneStore,  // store only once
    OneBlock,  // store and load in one basic block
    General,   // other than above
    Necessary, // cannot convert to register
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
            // TODO
            // for Necessary type
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
        let mut sthash: HashMap<Label, SsaInstrOp> = HashMap::new();
        for bb in &mut func.bls {
            let mut st_onebb_hash: HashMap<Label, SsaInstrOp> = HashMap::new();
            for instr in &mut bb.instrs {
                use SsaInstrOp::*;
                match &instr.op {
                    Assign(vty, v, rhs) if matches!(&rhs.op, Loadw(..)) => {
                        let vne = rhs.getld_vn();
                        if let Some(sop) = st_onebb_hash.get(vne) {
                            instr.op =
                                Assign(*vty, v.clone(), Box::new(SsaInstr::new(sop.clone())));
                            continue;
                        }
                        if MemToregAlloca::eztype(m2rinfo, vne) {
                            instr.op = Assign(
                                *vty,
                                v.clone(),
                                Box::new(SsaInstr::new(sthash.get(vne).unwrap().clone())),
                            );
                        }
                    }
                    Storew(fco, var) => {
                        st_onebb_hash.insert(var.name, SsaInstrOp::Src(fco.clone()));
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
            st_onebb_hash.clear();
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PhiNode(&'static str);

pub fn mem2reg(spg: &mut SsaProgram) {
    // insert Phi node to dominator frontier of every alloca defblock.
    for func in &mut spg.funcs {
        let mut insert_phi_bbs: Vec<Vec<PhiNode>> =
            vec![vec![]; func.cfg.as_deref().unwrap().graph.len()];
        for (_, value) in func.m2rinfo.iter() {
            for defbb in &value.defbbs {
                let domfs = &func.bls[*defbb].domfros;
                for _ in domfs.into_iter().map(|domf| {
                    if *domf >= insert_phi_bbs.capacity() {
                        panic!("insert_phi_bbs capacity error.");
                    }
                    if !insert_phi_bbs[*domf].contains(&PhiNode(value.name)) {
                        insert_phi_bbs[*domf].push(PhiNode(value.name));
                    }
                }) {}
            }
        }
        for bb in &mut func.bls {
            if bb.id >= insert_phi_bbs.capacity() {
                panic!("insert_phi_bbs capacity error2.");
            }
            for _ in insert_phi_bbs[bb.id].iter().map(|phin| {
                let phi_instr =
                    SsaInstr::new_all(SsaInstrOp::Phi(Some(phin.0), vec![]), true, bb.lb);
                let freshn = new_freshnum_phi();
                let varlb = format!("z#_{}", freshn);
                FRESH_NUM_MAP
                    .lock()
                    .unwrap()
                    .insert(freshn, Box::leak(varlb.into_boxed_str()));
                let var = Var::new(
                    FRESH_NUM_MAP.lock().unwrap().get(&freshn).unwrap(),
                    VarType::Word,
                    nextfreshregister(),
                );
                bb.instrs.insert(
                    0,
                    SsaInstr::new_all(
                        SsaInstrOp::Assign(ValueType::Word, var, Box::new(phi_instr)),
                        true,
                        bb.lb,
                    ),
                )
            }) {}
        }
    }
    // convert target load to src register
    for func in &mut spg.funcs {
        let mut walked_bbs = vec![false; func.cfg.as_ref().unwrap().graph.len()];
        let target_id = 0;
        walked_bbs[0] = true;
        let incoming_nodes = HashMap::new();
        let alloca_newvar_hash = HashMap::new();
        walk_bb(
            incoming_nodes,
            alloca_newvar_hash,
            &func.cfg,
            &mut func.bls,
            &mut walked_bbs,
            target_id,
        )
    }
    // delete unneccessary alloca and store
    for func in &mut spg.funcs {
        let m2rinfo = &func.m2rinfo;
        for bb in &mut func.bls {
            bb.instrs = bb
                .instrs
                .iter()
                .filter(|&isr| {
                    use MemToregType::*;
                    use SsaInstrOp::*;
                    match &isr.op {
                        Alloc4(alloca_var, _) | Storew(_, alloca_var) => {
                            match m2rinfo.get(alloca_var.name).unwrap().ty.unwrap() {
                                OneStore | OneBlock | General => false,
                                Necessary => true,
                            }
                        }
                        _ => true,
                    }
                })
                .cloned()
                .collect::<Vec<SsaInstr>>();
        }
    }
}

fn walk_bb(
    mut incoming_nodes: HashMap<Label, Vec<(Label, FirstClassObj)>>,
    mut alloca_newvar_hash: HashMap<(BBLabel, Label), Var>,
    cfg: &Option<Box<ControlFlowGraph>>,
    bbs: &mut Vec<SsaBlock>,
    walked_bbs: &mut Vec<bool>,
    target_id: usize,
) {
    let tbb = &mut bbs[target_id];
    for isr in &mut tbb.instrs {
        use SsaInstrOp::*;
        match &isr.op {
            Storew(fco, var) => {
                let alloca_label = var.name;
                incoming_nodes.insert(alloca_label, vec![(tbb.lb, fco.clone())]);
            }
            Assign(vty, var, rhs) if matches!(&rhs.op, Loadw(_)) => {
                let alloca_label = rhs.getld_vn();
                let phi_var_opt = alloca_newvar_hash.get(&(tbb.lb, alloca_label));
                if let Some(phi_var) = phi_var_opt {
                    isr.op = Assign(
                        *vty,
                        var.clone(),
                        Box::new(SsaInstr::new_all(
                            SsaInstrOp::Src(FirstClassObj::Variable(phi_var.clone())),
                            rhs.living,
                            rhs.bblb,
                        )),
                    );
                }
            }
            Assign(vty, var, rhs) if matches!(&rhs.op, Phi(_, _)) => {
                let alloca_label = rhs.getalloca_label();
                if !alloca_newvar_hash.contains_key(&(tbb.lb, alloca_label)) {
                    alloca_newvar_hash.insert((tbb.lb, alloca_label), var.clone());
                }
                let mut incoming_fcos = rhs.getincoming_fcos();
                let add_incoming_fcos = incoming_nodes.get_into(alloca_label);
                if let None = add_incoming_fcos {
                    continue;
                }
                incoming_fcos.append(&mut add_incoming_fcos.unwrap());
                isr.op = Assign(
                    *vty,
                    var.clone(),
                    Box::new(SsaInstr::new_all(
                        Phi(Some(alloca_label), incoming_fcos),
                        rhs.living,
                        rhs.bblb,
                    )),
                );
            }
            _ => {}
        }
    }
    walked_bbs[target_id] = true;
    let graphs = cfg.as_ref().unwrap();
    for come_id in &graphs.rgraph[target_id] {
        walked_bbs[target_id] &= walked_bbs[*come_id];
    }
    for next_id in &graphs.graph[target_id] {
        if walked_bbs[*next_id] {
            continue;
        }
        walk_bb(
            incoming_nodes.clone(),
            alloca_newvar_hash.clone(),
            cfg,
            bbs,
            walked_bbs,
            *next_id,
        );
    }
}
