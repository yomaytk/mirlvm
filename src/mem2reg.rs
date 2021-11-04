use super::dominators::ControlFlowGraph;
use super::parser::{
    nextfreshregister, FirstClassObj, SsaBlock, SsaInstr, SsaInstrOp, SsaProgram, ValueType, Var,
    VarType,
};
use super::*;
use once_cell::sync::Lazy;
use std::collections::VecDeque;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
extern crate rand;
use rand::seq::SliceRandom;


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
                let mut inserted_domfs = func.bls[*defbb].domfros.clone();
                while let Some(domf) = inserted_domfs.pop() {
                    if domf >= insert_phi_bbs.capacity() {
                        panic!("insert_phi_bbs capacity error.");
                    }
                    if !insert_phi_bbs[domf].contains(&PhiNode(value.name)) {
                        insert_phi_bbs[domf].push(PhiNode(value.name));
                        let mut continued_domfs = func.bls[domf].domfros.clone();
                        inserted_domfs.append(&mut continued_domfs);
                    }
                }
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
        let mut able_reach_nodes = vec![HashSet::new(); func.cfg.as_ref().unwrap().graph.len()];
        let mut edge_nums = 0;
        for nodes in &func.cfg.as_ref().unwrap().graph {
            edge_nums += nodes.len();
        }
        for snode in 0..able_reach_nodes.len() {
            let mut walked = vec![usize::MAX; able_reach_nodes.len()];
            let mut dequeue = VecDeque::new();
            dequeue.push_back(snode);
            while let Some(node) = dequeue.pop_front() {
                for enode in &func.cfg.as_ref().unwrap().graph[node] {
                    if walked[*enode] != usize::MAX {
                        continue;
                    }
                    dequeue.push_back(*enode);
                    walked[*enode] = *enode;
                }
            }
            able_reach_nodes[snode] = walked
                .iter()
                .filter(|&node| *node != usize::MAX)
                .cloned()
                .collect::<HashSet<usize>>();
        }
        let target_id = 0;
        let incoming_nodes = HashMap::new();
        let alloca_newvar_hash = HashMap::new();
        let mut current_reached_nodes = HashSet::new();
        let mut reached_edges = HashSet::new();
        walk_bb(
            incoming_nodes,
            alloca_newvar_hash,
            &func.cfg,
            &mut func.bls,
            &able_reach_nodes,
            &mut current_reached_nodes,
            &mut reached_edges,
            &edge_nums,
            target_id,
            false,
        );
        println!("{:?}", reached_edges);
    }
    // delete unneccessary alloca and store
    for func in &mut spg.funcs {
        let m2rinfo = &func.m2rinfo;
        use MemToregType::*;
        use SsaInstrOp::*;
        for bb in &mut func.bls {
            bb.instrs = bb
                .instrs
                .iter()
                .filter(|&isr| match &isr.op {
                    Alloc4(alloca_var, _) | Storew(_, alloca_var) => {
                        match m2rinfo.get(alloca_var.name).unwrap().ty.unwrap() {
                            OneStore | OneBlock | General => false,
                            Necessary => true,
                        }
                    }
                    _ => true,
                })
                .cloned()
                .collect::<Vec<SsaInstr>>();
        }
    }
}

fn walk_bb(
    mut incoming_nodes: HashMap<Label, Vec<(Label, FirstClassObj)>>,
    mut alloca_newvar_hash: HashMap<Label, Var>,
    cfg: &Option<Box<ControlFlowGraph>>,
    bbs: &mut Vec<SsaBlock>,
    able_reach_nodes: &Vec<HashSet<usize>>,
    current_reached_nodes: &mut HashSet<usize>,
    reached_edges: &mut HashSet<(usize, usize)>,
    edge_nums: &usize,
    target_id: usize,
    mut new_edge: bool,
) {
    let tbb = &mut bbs[target_id];
    use SsaInstrOp::*;
    for isr in &mut tbb.instrs {
        match &isr.op {
            Storew(fco, var) => {
                let alloca_label = var.name;
                incoming_nodes.insert(alloca_label, vec![(tbb.lb, fco.clone())]);
            }
            Assign(vty, var, rhs) if matches!(&rhs.op, Loadw(_)) => {
                let alloca_label = rhs.getld_vn();
                let phi_var_opt = alloca_newvar_hash.get(alloca_label);
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
                alloca_newvar_hash.insert(alloca_label, var.clone());
                let mut incoming_fcos = rhs.getincoming_fcos();
                let add_incoming_fcos = incoming_nodes.get_into(alloca_label);
                if let None = add_incoming_fcos {
                    continue;
                }
                'outer1: for (tbb_lb1, fco1) in add_incoming_fcos.unwrap() {
                    for (tbb_lb2, _) in &incoming_fcos {
                        if &tbb_lb1 == tbb_lb2 {
                            continue 'outer1;
                        }
                    }
                    incoming_fcos.push((tbb_lb1, fco1));
                }
                // new var assigend phi function
                incoming_nodes.insert(
                    alloca_label,
                    vec![(tbb.lb, FirstClassObj::Variable(var.clone()))],
                );
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
    
    if new_edge {
        current_reached_nodes.clear();
    }

    let mut next_nodes = vec![];
    'outer2: loop {
        if cfg.as_ref().unwrap().graph[target_id].len() == 0 {
            break;
        }
        next_nodes.clone_from(&cfg.as_ref().unwrap().graph[target_id]);
        next_nodes.shuffle(&mut rand::thread_rng());
        for next_id in &next_nodes {
            if able_reach_nodes[target_id].is_subset(current_reached_nodes) {
                break 'outer2;
            }
            if reached_edges.contains(&(target_id, *next_id)) {
                new_edge = false;
            } else {
                reached_edges.insert((target_id, *next_id));
                new_edge = true;
            }
            current_reached_nodes.insert(*next_id);
            walk_bb(
                incoming_nodes.clone(),
                alloca_newvar_hash.clone(),
                cfg,
                bbs,
                able_reach_nodes,
                current_reached_nodes,
                reached_edges,
                edge_nums,
                *next_id,
                new_edge,
            );
        }
    }
}
