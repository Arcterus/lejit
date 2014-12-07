use std::{u8, u16, u32, u64};
use super::{Compilable, Opcode, Register, Jit, JitFunction, JitOp, JitReg};

//static REXW: u8 = 0x48;
static RET: u8 = 0xc3;

#[experimental]
impl<'a> Compilable<'a> for JitFunction<'a> {
   fn compile(&self, jit: &'a Jit<'a>, pos: uint) -> Vec<u8> {
      let mut vec = vec!();
      let mut pos = pos;
      for op in self.ops.iter() {
         let comp = op.compile(jit, pos);
         println!("{}", comp);
         pos += comp.len();
         vec.extend(comp.into_iter());
      }
      vec
   }
}

#[experimental]
impl<'a> Opcode for JitOp<'a> {
   fn len(&self) -> uint {
      match *self {
         JitOp::Addri(_, imm) | JitOp::Subri(_, imm) =>
            if needed_bytes(imm) == 1 {
               4
            } else {
               7 // TODO: account for 4
            },
         JitOp::Mulri(reg, imm) => encode_mulri(reg, imm).len(), /* mul needs to be variable-length here, so... */
         JitOp::Mulrr(reg1, reg2) => encode_mulrr(reg1, reg2).len(),
         JitOp::Divri(reg, imm) => encode_divri(reg, imm).len(),
         JitOp::Divrr(reg1, reg2) => encode_divrr(reg1, reg2).len(),
         JitOp::Movrr(reg1, reg2) => {
            let rex = if encode_rex(reg1, Some(reg2), false) == 0b01000000 { 0 } else { 1 };
            3 + rex
         }
         JitOp::Movri(reg, imm) => {
            let rex = if encode_rex(reg, None, false) == 0b01000000 { 0 } else { 1 };
            rex + match imm {
               1 | 2 | 3 => 5,
               _ => 9
            }
         },
         JitOp::Pushr(reg) => 1 + if encode_rex(reg, None, false) == 0b01000000 { 0 } else { 1 },
         JitOp::Popr(reg) => 1 + if encode_rex(reg, None, false) == 0b01000000 { 0 } else { 1 },
         JitOp::Call(_) => 5,
         JitOp::Ret => 1
      }
   }
}

#[experimental]
impl<'a> Compilable<'a> for JitOp<'a> {
   fn compile(&self, jit: &'a Jit<'a>, pos: uint) -> Vec<u8> {
      match *self {
         JitOp::Addri(reg, imm) => encode_addri(reg, imm),
         JitOp::Subri(reg, imm) => encode_subri(reg, imm),
         JitOp::Mulri(reg, imm) => encode_mulri(reg, imm),
         JitOp::Mulrr(reg1, reg2) => encode_mulrr(reg1, reg2),
         JitOp::Divri(reg, imm) => encode_divri(reg, imm),
         JitOp::Divrr(reg1, reg2) => encode_divrr(reg1, reg2),
         JitOp::Movrr(reg1, reg2) => encode_movrr(reg1, reg2),
         JitOp::Movri(reg, imm) => encode_movri(reg, imm),
         JitOp::Pushr(reg) => encode_pushr(reg),
         JitOp::Popr(reg) => encode_popr(reg),
         JitOp::Call(name) => encode_call(jit.find_function(name), pos),
         JitOp::Ret => vec!(RET),
         //_ => unimplemented!() // TODO: implement all ops
      }
   }
}

#[inline]
fn encode_rex(reg: JitReg, reg2: Option<JitReg>, w_field: bool) -> u8 {
   // TODO: handle SIB if needed
   let mut res = if w_field {
      0b01001000
   } else {
      0b01000000
   };
   if reg.extended() {
      res += if reg2.is_some() {
         1 << 2
      } else {
         1
      };
   }
   if reg2.is_some() {
      if reg2.unwrap().extended() {
         res += 1;
      }
   }
   res
}

#[inline]
fn encode_addri(reg: JitReg, imm: u64) -> Vec<u8> {
   let rex = encode_rex(reg, None, true);
   match needed_bytes(imm) {
      1 => vec!(rex, 0x83, (0b11 << 6) + reg.to_real_reg(), imm as u8),
      2 | 3 => vec!(rex, 0x81, (0b11 << 6) + reg.to_real_reg(), imm as u8, (imm >> 8) as u8, (imm >> 16) as u8, (imm >> 24) as u8),
      4 => panic!(), // TODO: should mov and then add
      _ => unreachable!()
   }
}

#[inline]
fn encode_subri(reg: JitReg, imm: u64) -> Vec<u8> {
   let rex = encode_rex(reg, None, true);
   match needed_bytes(imm) {
      1 => vec!(rex, 0x83, (0b11 << 6) + (0b101 << 3) + reg.to_real_reg(), imm as u8),
      2 | 3 => vec!(rex, 0x81, (0b11 << 6) + (0b101 << 3) + reg.to_real_reg(), imm as u8, (imm >> 8) as u8, (imm >> 16) as u8, (imm >> 24) as u8),
      4 => panic!(), // TODO: should mov and then add
      _ => unreachable!()
   }
}

#[inline]
fn encode_mul_div_rr(reg: JitReg, reg2: JitReg, special: u8) -> Vec<u8> {
   let mut res = vec!();
   if reg != JitReg::R1 { /* rax */
      res.extend(encode_pushr(JitReg::R1).into_iter());
      res.extend(encode_movrr(JitReg::R1, reg).into_iter());
   }
   if reg2 != JitReg::R3 { /* rsi */
      res.extend(encode_pushr(JitReg::R3).into_iter());
      res.extend(encode_movrr(JitReg::R3, reg2).into_iter());
   }
   res.push_all(&[encode_rex(JitReg::R3, None, true), 0xf7,
                             (0b11 << 6) + (special << 3) + JitReg::R3.to_real_reg()]);
   if reg2 != JitReg::R3 { /* rsi */
      res.extend(encode_popr(JitReg::R3).into_iter());
   }
   if reg != JitReg::R1 { /* rax */
      res.extend(encode_movrr(reg, JitReg::R1).into_iter());
      res.extend(encode_popr(JitReg::R1).into_iter());
   }
   res
}

#[inline]
fn encode_mulrr(reg: JitReg, reg2: JitReg) -> Vec<u8> {
   encode_mul_div_rr(reg, reg2, 0b100)
}

#[inline]
fn encode_mulri(reg: JitReg, imm: u64) -> Vec<u8> {
   let mut res = vec!();
   let immreg =
      if reg == JitReg::R3 { /* rsi */
         JitReg::R5 /* rcx */
      } else {
         JitReg::R3
      };
   res.extend(encode_pushr(immreg).into_iter());
   res.extend(encode_movri(immreg, imm).into_iter());
   res.extend(encode_mulrr(reg, immreg).into_iter());
   res.extend(encode_popr(immreg).into_iter());
   res
}

#[inline]
fn encode_divrr(reg1: JitReg, reg2: JitReg) -> Vec<u8> {
   encode_mul_div_rr(reg1, reg2, 0b110)
}

#[inline]
fn encode_divri(reg: JitReg, imm: u64) -> Vec<u8> {
   let mut res = vec!();
   let immreg =
      if reg == JitReg::R3 { /* rsi */
         JitReg::R5 /* rcx */
      } else {
         JitReg::R3
      };
   res.extend(encode_pushr(immreg).into_iter());
   res.extend(encode_movri(immreg, imm).into_iter());
   res.extend(encode_divrr(reg, immreg).into_iter());
   res.extend(encode_popr(immreg).into_iter());
   res
}

#[inline]
fn encode_pushr(reg: JitReg) -> Vec<u8> {
   let rex = encode_rex(reg, None, true);
   if rex == 0b01001000 {
      vec!(0x50 + reg.to_real_reg())
   } else {
      vec!(rex, 0x50 + reg.to_real_reg())
   }
}

#[inline]
fn encode_popr(reg: JitReg) -> Vec<u8> {
   let rex = encode_rex(reg, None, true);
   if rex == 0b01001000 {
      vec!(0x58 + reg.to_real_reg())
   } else {
      vec!(rex, 0x58 + reg.to_real_reg())
   }
}

#[inline]
fn encode_movrr(reg1: JitReg, reg2: JitReg) -> Vec<u8> {
   vec!(encode_rex(reg1, Some(reg2), true), 0x89,
                   (0b11 << 6) + (reg2.to_real_reg() << 3) + reg1.to_real_reg() as u8)
}

#[inline]
fn encode_movri(reg: JitReg, imm: u64) -> Vec<u8> {
   let rex = encode_rex(reg, None, false);
   match needed_bytes(imm) {
      1 | 2 | 3 => {
         if rex == 0b01000000 {
            vec!(0xb8 + reg.to_real_reg(), imm as u8, (imm >> 8) as u8,
                 (imm >> 16) as u8, (imm >> 24) as u8)
         } else {
            vec!(rex, 0xb8 + reg.to_real_reg(), imm as u8, (imm >> 8) as u8,
                 (imm >> 16) as u8, (imm >> 24) as u8)
         }
      }
      4 => vec!(rex + (1 << 3), 0xb8 + reg.to_real_reg(), imm as u8,
                (imm >> 8) as u8, (imm >> 16) as u8, (imm >> 24) as u8,
                (imm >> 32) as u8, (imm >> 40) as u8, (imm >> 48) as u8,
                (imm >> 56) as u8),
      _ => unreachable!()
   }
}

#[inline]
fn encode_call<'a>(func: Option<&'a JitFunction<'a>>, pos: uint) -> Vec<u8> {
   match func {
      Some(func) => {
         let mut pos = func.label.pos as i32 - pos as i32;
         if pos > 0 {
            pos -= 5;
         }
         vec!(0xe8, pos as u8, (pos >> 8) as u8, (pos >> 16) as u8, (pos >> 24) as u8)
      }
      None => panic!() // XXX: fix
   }
}

#[inline]
fn needed_bytes(num: u64) -> uint {
   if num <= u8::MAX as u64 {
      1
   } else if num <= u16::MAX as u64 {
      2
   } else if num <= u32::MAX as u64 {
      3
   } else if num <= u64::MAX {
      4
   } else {
      unreachable!()
   }
}

#[experimental]
impl Register for JitReg {
   fn to_real_reg(&self) -> u8 {
      match *self {
         JitReg::R1  => 0b000, /* rax */
         JitReg::R2  => 0b111, /* rdi */
         JitReg::R3  => 0b110, /* rsi */
         JitReg::R4  => 0b010, /* rdx */
         JitReg::R5  => 0b001, /* rcx */
         JitReg::R6  => 0b000, /* r8  */
         JitReg::R7  => 0b001, /* r9  */
         JitReg::R8  => 0b011, /* rbx */
         JitReg::R9  => 0b010, /* r10 */
         JitReg::R10 => 0b011, /* r11 */
         JitReg::R11 => 0b100, /* r12 */
         JitReg::R12 => 0b101, /* r13 */
         JitReg::R13 => 0b110, /* r14 */
         JitReg::R14 => 0b111, /* r15 */
         JitReg::SP  => 0b100, /* rsp */
         JitReg::BP  => 0b101  /* rbp */
      }
   }

   fn extended(&self) -> bool {
      match *self {
         JitReg::R6 | JitReg::R7 | JitReg::R9 | JitReg::R10 | JitReg::R11 |
               JitReg::R12 | JitReg::R13 | JitReg::R14 => true,
         _ => false
      }
   }
}
