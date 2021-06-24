use super::lowir::{LowIrInstr, LowIrProgram, RegorNum};
use super::*;
use super::lexer::Binop;

const REGQUANTITY: usize = 13;
const NORMALREGQUANTITY: usize = 7;
pub static X64_REG64: [&str; REGQUANTITY] = [
    "r10", "r11", "rbx", "r12", "r13", "r14", "r15", "rdi", "rsi", "rdx", "rcx", "r8", "r9",
];
pub static X64_REG32: [&str; REGQUANTITY] = [
    "r10d", "r11d", "ebx", "r12d", "r13d", "r14d", "r15d", "edi", "esi", "edx", "ecx", "r8d", "r9d",
];
pub static X64_REG8: [&str; REGQUANTITY] = [
    "r10b", "r11b", "bl", "r12b", "r13b", "r14b", "r15b", "dil", "sil", "dl", "cl", "r8b", "r9b",
];
fn selreg(r: lowir::Register) -> &'static str {
    if r.regsize == 4 {
        return X64_REG32[r.rr as usize];
    } else if r.regsize == 8 {
        return X64_REG64[r.rr as usize];
    }
    panic!("undefined register size.");
}

fn selargreg(size: usize, index: usize) -> &'static str {
    if size == 4 {
        return X64_REG32[index + NORMALREGQUANTITY];
    } else if size == 8 {
        return X64_REG64[index + NORMALREGQUANTITY];
    }
    panic!("undefined argument register size.");
}

fn memoryaccesssize(r: lowir::Register) -> &'static str {
    if r.regsize == 4 {
        return "DWORD PTR";
    } else if r.regsize == 8 {
        return "QWORD PTR";
    }
    panic!("memoryaccesssize error.");
}

fn selrax(size: usize) -> &'static str {
    if size == 4 {
        return "eax";
    } else if size == 8 {
        return "rax";
    }
    panic!("selrax error.");
}

pub fn gen_x64code(lirpg: LowIrProgram) {
    print!(".intel_syntax noprefix\n");
    print!(".text\n");
    print!(".globl main\n");

    for func in lirpg.funcs {
        let stmsize = (func.framesize + 15) / 16 * 16;
        print!("{}:\n", func.lb);
        print!("\tpush rbp\n");
        print!("\tmov rbp, rsp\n");
        if stmsize > 0 {
            print!("\tsub rsp, {}\n", stmsize);
        }
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
                        print!("\tmov {}, {}\n", selrax(r.regsize as usize), selreg(r));
                        if stmsize > 0 {
                            print!("\tadd rsp, {}\n", stmsize);
                        }
                        print!("\tpop rbp\n");
                        print!("\tret\n");
                    }
                    Storewreg(r, offset) => {
                        print!(
                            "\tmov {} [rbp-{}], {}\n",
                            memoryaccesssize(r),
                            offset,
                            selreg(r)
                        );
                    }
                    Storewnum(num, offset) => {
                        print!("\tmov DWORD PTR [rbp-{}], {}\n", offset, num);
                    }
                    Loadw(r, offset) => {
                        print!(
                            "\tmov {}, {} [rbp-{}]\n",
                            selreg(r),
                            memoryaccesssize(r),
                            offset
                        );
                    }
                    Bop(binop, r1, r2) => {
                        let op = match binop {
                            Binop::Add => "add",
                            Binop::Sub => "sub"
                        };
                        match r2 {
                            RegorNum::Reg(r) => { print!("\t{} {}, {}\n", op, selreg(r1), selreg(r)); }
                            RegorNum::Num(num) => { print!("\t{} {}, {}\n", op, selreg(r1), num); }
                        }
                    }
                    Call(r1, lb, args, mut usedrs) => {
                        for i in &usedrs {
                            print!("\tpush {}\n", X64_REG64[*i]);
                        }
                        for i in 0..args.len() {
                            match args[i] {
                                RegorNum::Reg(r) => {
                                    print!(
                                        "\tmov {}, {}\n",
                                        selargreg(r.regsize as usize, i),
                                        selreg(r)
                                    );
                                }
                                RegorNum::Num(num) => {
                                    print!("\tmov {}, {}\n", selargreg(4, i), num);
                                }
                            }
                        }
                        print!("\tcall {}\n", lb);
                        usedrs.reverse();
                        for i in usedrs {
                            print!("\tpop {}\n", X64_REG64[i]);
                        }
                        print!("\tmov {}, {}\n", selreg(r1), selrax(r1.regsize as usize));
                    }
                    Ceqw(r1, r2, rorn) => {
                        match rorn {
                            RegorNum::Reg(r3) => {
                                print!("\tcmp {}, {}\n", selreg(r2), selreg(r3));
                            }
                            RegorNum::Num(num) => {
                                print!("\tcmp {}, {}\n", selreg(r2), num);
                            }
                        }
                        print!("\tsete {}\n", X64_REG8[r1.rr as usize]);
                        print!("\tmovzb {}, {}\n", X64_REG64[r1.rr as usize], X64_REG8[r1.rr as usize]);
                    }
                    Jnz(r1, lb1, lb2) => {
                        print!("\tcmp {}, 0\n", selreg(r1));
                        print!("\tjne {}\n", lb1);
                        print!("\tjmp {}\n", lb2);
                    }
                    Jmp(lb) => {
                        print!("\tjmp {}\n", lb);
                    }
                }
            }
        }
    }
}
