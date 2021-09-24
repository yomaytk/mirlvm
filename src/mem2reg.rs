use super::dominators::*;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemToregType {
    OneStore, // store only once
    OneBlock, // store and load in one basic block
    General,  // othre than above
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
}
