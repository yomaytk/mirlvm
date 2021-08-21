use super::parser::*;
use super::*;
use super::parser::SsaInstrOp::*;
use std::collections::HashMap;

pub fn removeuselessinstr(ssapg: &mut SsaProgram) {
    let mut defliveisrs = vec![];
    let mut newvarinstrs = HashMap::new();
    let mut storeinstrs = HashMap::new();
    let mut compinstrs = vec![];
    let mut brinstrs = vec![];
    for func in &mut ssapg.funcs {
        for bl in &mut func.bls {
            for ssaisr in &mut bl.instrs {
                match &ssaisr.op {
                    Ret(..) | Call(..) | Jmp(..) => {
                        ssaisr.living = true;
                        defliveisrs.push(ssaisr);
                        bl.liveinstrcnt += 1;
                    }
                    Assign(_, var, ..) | Alloc4(var, _) => {
                        newvarinstrs.insert(var.name, ssaisr);
                    }
                    Storew(_, var) => {
                        storeinstrs.insert(var.name, ssaisr);
                    }
                    Comp(..) => {
                        compinstrs.push(ssaisr);
                    }
                    Jnz(..) => {
                        brinstrs.push(ssaisr);
                    }
                    Phi(..) => {}
                    _ => {
                        panic!("removeuserlessinstr error.");
                    }
                }
            }
        }
    }
    assert_eq!(compinstrs.len(), brinstrs.len());
    let mut pbr = brinstrs.iter();
    let mut compbrinstrs = vec![];
    for pair in compinstrs.into_iter().map(|comp| { (comp, pbr.next().unwrap()) }) {
        compbrinstrs.push(pair);
    }
    while let Some(ssaisr) = defliveisrs.pop() {
        let varnames = findvarsininstr(&ssaisr);
        // var is living defined in ssaisr
        for varn in varnames {
            if let Some(isr) = newvarinstrs.remove(varn) {
                isr.living = true;
                defliveisrs.push(isr);
            }
            if let Some(isr) = storeinstrs.remove(varn) {
                isr.living = true;
                defliveisrs.push(isr);
            }
        }
    }
    // for (_comp, _br) in compbrinstrs {
        
    // }
}

fn findvarsininstr(ssaisr: &SsaInstr) -> Vec<VarName> {
    let mut varnames = vec![];
    match &ssaisr.op {
        Ret(fco) => {
            if let FirstClassObj::Variable(var) = fco {
                varnames.push(var.name);
            }
        }
        Assign(.., ssainstr) => {
            varnames = [varnames, findvarsininstr(&ssainstr)].concat();
        }
        Loadw(var) | Jnz(var, ..)=> {
            varnames.push(var.name);
        }
        Storew(fco, _) => {
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
        Call(.., fcos) => {
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