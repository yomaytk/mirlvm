use super::lexer::Binop;
use super::lowir::{LowIrInstr, LowIrProgram, RegorNum, Register};
use super::parser::FirstClassObj;
use super::*;
use parser::CompOp;

extern crate rand;
use rand::seq::SliceRandom;

const REGQUANTITY: usize = 13;
pub const NORMALREGQUANTITY: usize = 7;

pub static X64_REG64: [&str; REGQUANTITY] = [
    "r10", "r11", "rbx", "r12", "r13", "r14", "r15", "rdi", "rsi", "rdx", "rcx", "r8", "r9",
];
pub static X64_REG32: [&str; REGQUANTITY] = [
    "r10d", "r11d", "ebx", "r12d", "r13d", "r14d", "r15d", "edi", "esi", "edx", "ecx", "r8d", "r9d",
];
pub static X64_REG8: [&str; REGQUANTITY] = [
    "r10b", "r11b", "bl", "r12b", "r13b", "r14b", "r15b", "dil", "sil", "dl", "cl", "r8b", "r9b",
];

fn selreg(r: &Register) -> &'static str {
    if r.regsize == 4 {
        return X64_REG32[r.rr as usize];
    } else if r.regsize == 8 {
        return X64_REG64[r.rr as usize];
    } else if r.regsize == 1 {
        // need fix
        return X64_REG32[r.rr as usize];
    }
    panic!("undefined register size. {:?}", r);
}

fn selargreg(size: usize, index: usize) -> &'static str {
    if size == 4 {
        return X64_REG32[index + NORMALREGQUANTITY];
    } else if size == 8 {
        return X64_REG64[index + NORMALREGQUANTITY];
    } else if size == 1 {
        // need fix
        return X64_REG32[index + NORMALREGQUANTITY];
    }
    panic!("undefined argument register size. register: index: {}, size: {}", index, size);
}

fn memoryaccesssize(r: &Register) -> &'static str {
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

fn movregreg(r1: &Register, r2: &Register) {
    if let Some(gl_lb) = r2.global {
        print!("\tmov {}, OFFSET FLAT:{}\n", X64_REG64[r1.rr as usize], gl_lb);
    } else {
        print!("\tmov {}, {}\n", selreg(r1), selreg(r2));
    }
}


const BASE_STR: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

fn gen_random_label(size: usize) -> String {
    let mut rng = &mut rand::thread_rng();
    String::from_utf8(
        BASE_STR.as_bytes()
            .choose_multiple(&mut rng, size)
            .cloned()
            .collect()
    ).unwrap()
}

fn gen_jmp_overflow(overflow_black_label: String) {
    print!("\tpushf\n");
    print!("\tmov r15d, [rsp]\n");
    print!("\tand r15d, 0x00000800\n");
    print!("\tcmp r15d, 0\n");
    print!("\tjne .{}\n", overflow_black_label);
    print!("\tpopf\n");
}

fn gen_overflow_block(num_overflow_label: String, overflow_black_label: String) {
    print!(".{}:\n", overflow_black_label);
    print!("\tmov edi, OFFSET FLAT:.{}\n", num_overflow_label);
    print!("\tmov eax, 0\n");
    print!("\tcall printf\n");
    print!("\tpopf\n");
    print!("\tpop rbp\n");
    print!("\tret\n");
}

pub fn gen_x64code(lirpg: LowIrProgram, secure_mode: bool) {
    print!(".intel_syntax noprefix\n");

    // data section
    print!(".data\n");
    print!("\n");
    let num_overflow_label = gen_random_label(100);
    let overflow_black_label = gen_random_label(50);
    if secure_mode {
        print!(".{}:\n", &num_overflow_label[..]);
        print!("\t.string \"execution error of integer overflow.\\n\"\n");
    }

    for gd in lirpg.gvs {
        print!("{}:\n", gd.lb);
        for eled in gd.dts {
            if let FirstClassObj::String(lb) = eled {
                print!(".LC{}:\n", -gd.frsn);
                print!("\t.string \"{}\"\n", lb);
            }
        }
    }

    print!("\n");

    // execution program section
    print!(".text\n");
    print!(".globl main\n");
    print!("\n");

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
                    Movenum(ref r, num) => {
                        print!("\tmov {}, {}\n", selreg(r), num);
                    }
                    Movereg(ref r1, ref r2) => {
                        movregreg(r1, r2);
                    }
                    Ret(ref r) => {
                        print!("\tmov {}, {}\n", selrax(r.regsize as usize), selreg(r));
                        if stmsize > 0 {
                            print!("\tadd rsp, {}\n", stmsize);
                        }
                        print!("\tpop rbp\n");
                        print!("\tret\n");
                    }
                    Storewreg(ref r, offset) => {
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
                    Loadw(ref r, offset) => {
                        print!(
                            "\tmov {}, {} [rbp-{}]\n",
                            selreg(r),
                            memoryaccesssize(r),
                            offset
                        );
                    }
                    Bop(binop, ref r1, ref r2) => {
                        let op = match binop {
                            Binop::Add => "add",
                            Binop::Sub => "sub",
                            Binop::Mul => "mul",
                        };
                        match r2 {
                            RegorNum::Reg(r) => {
                                if op == "add" || op == "sub" {
                                    print!("\t{} {}, {}\n", op, selreg(r1), selreg(r));
                                } else {
                                    assert_eq!(op, "mul");
                                    print!("\tmov {}, {}\n", selrax(r.regsize as usize), selreg(r));
                                    print!("\timul {}\n", selreg(r1));
                                    print!("\tmov {}, {}\n", selreg(r1), selrax(r.regsize as usize))
                                }
                            }
                            RegorNum::Num(num) => {
                                assert!(op == "add" || op == "sub");
                                print!("\t{} {}, {}\n", op, selreg(r1), num);
                            }
                        }
                        if secure_mode {
                            if op == "add" || op == "mul" {
                                gen_jmp_overflow(overflow_black_label.clone());
                            }
                        }
                    }
                    Call(ref r1, lb, ref args, mut usedrs) => {
                        for i in &usedrs {
                            print!("\tpush {}\n", X64_REG64[*i]);
                        }
                        for i in 0..args.len() {
                            match args[i] {
                                RegorNum::Reg(ref r) => {
                                    if let Some(gl_lb) = r.global {
                                        print!(
                                            "\tmov {}, OFFSET FLAT:{}\n",
                                            selargreg(r.regsize as usize, i),
                                            gl_lb
                                        );
                                    } else {
                                        print!(
                                            "\tmov {}, {}\n",
                                            selargreg(r.regsize as usize, i),
                                            selreg(r)
                                        );
                                    }
                                }
                                RegorNum::Num(num) => {
                                    print!("\tmov {}, {}\n", selargreg(4, i), num);
                                }
                            }
                        }
                        if lb == "printf" {
                            print!("\tmov eax, 0\n");
                        }
                        print!("\tcall {}\n", lb);
                        usedrs.reverse();
                        for i in usedrs {
                            print!("\tpop {}\n", X64_REG64[i]);
                        }
                        if r1.regsize > 0 {
                            print!("\tmov {}, {}\n", selreg(r1), selrax(r1.regsize as usize));
                        }
                    }
                    Comp(op, ref r1, ref r2, ref rorn) => {
                        match rorn {
                            RegorNum::Reg(r3) => {
                                print!("\tcmp {}, {}\n", selreg(r2), selreg(r3));
                            }
                            RegorNum::Num(num) => {
                                print!("\tcmp {}, {}\n", selreg(r2), num);
                            }
                        }
                        match op {
                            CompOp::Ceqw => {
                                print!("\tsete {}\n", X64_REG8[r1.rr as usize]);
                            }
                            CompOp::Csltw => {
                                print!("\tsetl {}\n", X64_REG8[r1.rr as usize]);
                            }
                        }
                        print!(
                            "\tmovzb {}, {}\n",
                            X64_REG64[r1.rr as usize], X64_REG8[r1.rr as usize]
                        );
                    }
                    Jnz(ref r1, lb1, lb2) => {
                        print!("\tcmp {}, 0\n", selreg(r1));
                        print!("\tjne {}\n", lb1);
                        print!("\tjmp {}\n", lb2);
                    }
                    Jmp(lb) => {
                        print!("\tjmp {}\n", lb);
                    }
                    LowNop => {
                        panic!("cannot reach this instr.");
                    }
                }
            }
        }
    }
    if secure_mode {
        gen_overflow_block(num_overflow_label, overflow_black_label);
    }
}
