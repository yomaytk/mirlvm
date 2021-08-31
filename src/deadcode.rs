use super::parser::SsaInstrOp::*;
use super::parser::*;
use super::*;
use std::collections::HashMap;

struct BlockInfos {
    pub lbids: HashMap<Label, usize>,
    pub livings: Vec<usize>,
    pub livcah: Vec<usize>,
}

impl BlockInfos {
    fn new() -> Self {
        Self {
            lbids: HashMap::new(),
            livings: vec![],
            livcah: vec![],
        }
    }
    fn newbl(&mut self, lb: Label, id: usize) {
        self.lbids.insert(lb, id);
        self.livings.push(0);
    }
    fn newlivbl(&mut self, lb: Label) {
        let id = self.lbids.get(lb).unwrap();
        self.livings[*id] = 1;
    }
    fn bb_is_empty(&self, lb: Label) -> usize {
        let id = self.lbids.get(lb).unwrap();
        self.livings[*id]
    }
    pub fn init_livcah(&mut self) {
        let mut c = 0;
        for d in &self.livings {
            c += d;
            self.livcah.push(c);
        }
    }
}

pub fn removeuselessinstr(ssapg: &mut SsaProgram) {
    let mut defliveisrs: Vec<&mut SsaInstr> = vec![];
    let mut nrmisrs: HashMap<&str, Vec<&mut SsaInstr>> = HashMap::new();
    let mut bbinfos = BlockInfos::new();
    let mut bid = 0;
    for func in &mut ssapg.funcs {
        for bb in &mut func.bls {
            // new basic block
            bbinfos.newbl(bb.lb, bid);
            for isr in &mut bb.instrs {
                isr.bblb = bb.lb;
                match &isr.op {
                    Ret(..) | Call(..) | Jmp(..) | Jnz(..) => {
                        isr.living = true;
                        defliveisrs.push(isr);
                        bbinfos.newlivbl(bb.lb);
                    }
                    Assign(_, var, ..) | Alloc4(var, _) | Storew(_, var) | Comp(_, var, ..) => {
                        if let Some(tis) = nrmisrs.get_mut(var.name) {
                            tis.push(isr);
                        } else {
                            nrmisrs.insert(var.name, vec![isr]);
                        }
                    }
                    Phi(..) => {}
                    _ => {
                        // panic!("removeuserlessinstr error.");
                    }
                }
            }
            bid += 1;
        }
    }
    // let mut jmpzs = vec![];
    while let Some(isr) = defliveisrs.pop() {
        let varnames = findvarsininstr(&isr);
        // let mut f = |hashs: &mut HashMap<&str, &mut SsaInstr>, varn: &str| {
        //     if let Some(isr2) = hashs.remove(varn) {
        //         isr2.living = true;
        //         bbinfos.newlivbl(isr2.bblb);
        //         defliveisrs.push(isr2);
        //     }
        // };
        // var is living defined in isr
        for varn in varnames {
            if let Some(isrs) = nrmisrs.remove(varn) {
                for isr2 in isrs {
                    isr2.living = true;
                    bbinfos.newlivbl(isr2.bblb);
                    defliveisrs.push(isr2);
                }
            }
        }
        // match isr.op {
        //     Jmp(..) | Jnz(..) => {
        //         jmpzs.push(isr);
        //     }
        //     _ => {}
        // }
    }
    // while let Some(jisr) = jmpzs.pop() {
    //     match jisr.op {
    //         Jmp(lb) => {
    //             jisr.living = bbinfos.bb_is_empty(lb) == 1;
    //         }
    //         Jnz(_, lb1, lb2) => {
    //             jisr.living = bbinfos.bb_is_empty(lb1) == 1 && bbinfos.bb_is_empty(lb2) == 1;
    //         }
    //         _ => {}
    //     }
    // }
}

fn findvarsininstr(isr: &SsaInstr) -> Vec<VarName> {
    let mut varnames = vec![];
    match &isr.op {
        Ret(fco) => {
            if let FirstClassObj::Variable(var) = fco {
                varnames.push(var.name);
            }
        }
        Assign(.., ssainstr) => {
            varnames = [varnames, findvarsininstr(&ssainstr)].concat();
        }
        Loadw(var) | Jnz(var, ..) => {
            varnames.push(var.name);
        }
        Storew(fco, var) => {
            varnames.push(var.name);
            if let FirstClassObj::Variable(var2) = fco {
                varnames.push(var2.name);
            }
        }
        Bop(_, fco1, fco2) => {
            if let FirstClassObj::Variable(var1) = fco1 {
                varnames.push(var1.name);
            }
            if let FirstClassObj::Variable(var2) = fco2 {
                varnames.push(var2.name);
            }
        }
        Call(.., fcos, _) => {
            for fco in fcos {
                if let FirstClassObj::Variable(var) = fco {
                    varnames.push(var.name);
                }
            }
        }
        Comp(_, var1, var2, fco) => {
            varnames.push(var1.name);
            varnames.push(var2.name);
            if let FirstClassObj::Variable(var3) = fco {
                varnames.push(var3.name);
            }
        }
        Phi(vs) => {
            for v in vs {
                if let FirstClassObj::Variable(var) = &v.1 {
                    varnames.push(var.name);
                }
            }
        }
        Jmp(_) | Alloc4(..) => {}
    }
    varnames
}
