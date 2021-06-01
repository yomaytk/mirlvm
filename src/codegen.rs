use super::*;
use super::lowir::{LowIrProgram, LowIrInstr};

const REGQUANTITY: usize = 7;
pub static x64_reg64: [&str; REGQUANTITY] = ["r10", "r11", "rbx", "r12", "r13", "r14", "r15"];
pub static x64_reg32: [&str; REGQUANTITY] = ["r10d", "r11d", "ebx", "r12d", "r13d", "r14d", "r15d"];

fn selreg(r: lowir::Register) -> &'static str {
    if r.regsize == 4 {
        return x64_reg32[r.rr as usize]
    } else if r.regsize == 8 {
        return x64_reg64[r.rr as usize]
    }
    panic!("undefined register size.");
}

fn memoryaccesssize(r: lowir::Register) -> &'static str {
    if r.regsize == 4 {
        return "DWORD PTR";
    } else if r.regsize == 8 {
        return "QWORD PTR";
    }
    panic!("memoryaccesssize error.");
}

pub fn gen_x64code(lirpg: LowIrProgram) {
    print!(".intel_syntax noprefix\n");
    print!(".text\n");
    print!(".globl main\n");
    
    for func in lirpg.funcs {
        print!("{}:\n", func.lb);
        print!("\tpush rbp\n");
        print!("\tmov rbp, rsp\n");
        for bb in func.rbbs {
            print!("{}:\n", bb.lb);
            for instr in bb.instrs {
                use LowIrInstr::*;
                match instr {
                    Movenum(r, num) => {
                        print!("\tmov {}, {}\n", selreg(r), num);
                    }
                    Movereg(r1, r2) => {
                        print!("\tmov {}, {}\n", selreg(r1), selreg(r2));
                    }
                    Ret(r) => {
                        print!("\tmov {}, {}\n", if r.regsize == 4 { "eax" } else if r.regsize == 8 { "rax" } else { panic!("undefined return register.") }, selreg(r));
                        print!("\tpop rbp\n");
                        print!("\tret\n");
                    }
                    Storewreg(r, offset) => {
                        print!("\tmov {} [rbp-{}], {}\n", memoryaccesssize(r), offset, selreg(r));
                    }
                    Storewnum(num, offset) => {
                        print!("\tmov DWORD PTR [rbp-{}], {}\n", offset, num);
                    }
                    Loadw(r, offset) => {
                        print!("\tmov {}, {} [rbp-{}]\n", selreg(r), memoryaccesssize(r), offset);
                    }
                    Add(r1, r2) => {
                        print!("\tadd {}, {}\n", selreg(r1), selreg(r2));
                    }
                }
            }
        }
    }
}