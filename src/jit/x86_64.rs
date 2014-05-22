use std::{u8, u16, u32, u64};
use super::{Compilable, Jit, JitFunction, JitOp, JitReg};

//static REXW: u8 = 0x48;
static RET: u8 = 0xc3;

impl<'a> Compilable<'a> for JitFunction<'a> {
	fn compile(&self, jit: &'a Jit<'a>, pos: uint) -> Vec<u8> {
		let mut vec = vec!();
		let mut pos = pos;
		for op in self.ops.iter() {
			let comp = op.compile(jit, pos);
			println!("{}", comp);
			pos += comp.len();
			vec.push_all_move(comp);
		}
		vec
	}
}

impl<'a> super::JitOp<'a> {
	pub fn len(&self) -> uint {
		match *self {
			super::Addri(_, imm) | super::Subri(_, imm) =>
				if needed_bytes(imm) == 1 {
					4
				} else {
					7 // TODO: account for 4
				},
			super::Mulri(reg, imm) => encode_mulri(reg, imm).len(), /* mul needs to be variable-length here, so... */
			super::Mulrr(reg1, reg2) => encode_mulrr(reg1, reg2).len(),
			super::Movrr(reg1, reg2) => {
				let rex = if encode_rex(reg1, Some(reg2), false) == 0b01000000 { 0 } else { 1 };
				3 + rex
			}
			super::Movri(reg, imm) => {
				let rex = if encode_rex(reg, None, false) == 0b01000000 { 0 } else { 1 };
				rex + match imm {
					1 | 2 | 3 => 5,
					_ => 9
				}
			},
			super::Pushr(reg) => 1 + if encode_rex(reg, None, false) == 0b01000000 { 0 } else { 1 },
			super::Popr(reg) => 1 + if encode_rex(reg, None, false) == 0b01000000 { 0 } else { 1 },
			super::Call(_) => 5,
			super::Ret => 1
		}
	}
}

impl<'a> Compilable<'a> for JitOp<'a> {
	fn compile(&self, jit: &'a Jit<'a>, pos: uint) -> Vec<u8> {
		match *self {
			super::Addri(reg, imm) => encode_addri(reg, imm),
			super::Subri(reg, imm) => encode_subri(reg, imm),
			super::Mulri(reg, imm) => encode_mulri(reg, imm),
			super::Mulrr(reg1, reg2) => encode_mulrr(reg1, reg2),
			super::Movrr(reg1, reg2) => encode_movrr(reg1, reg2),
			super::Movri(reg, imm) => encode_movri(reg, imm),
			super::Pushr(reg) => encode_pushr(reg),
			super::Popr(reg) => encode_popr(reg),
			super::Call(name) => encode_call(jit.find_function(name), pos),
			super::Ret => vec!(RET),
			//_ => fail!() // TODO: implement all ops
		}
	}
}

#[inline(always)]
fn encode_rex(reg: super::JitReg, reg2: Option<super::JitReg>, w_field: bool) -> u8 {
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

#[inline(always)]
fn encode_addri(reg: super::JitReg, imm: u64) -> Vec<u8> {
	let rex = encode_rex(reg, None, true);
	match needed_bytes(imm) {
		1 => vec!(rex, 0x83, (0b11 << 6) + reg.to_real_reg(), imm as u8),
		2 | 3 => vec!(rex, 0x81, (0b11 << 6) + reg.to_real_reg(), imm as u8, (imm >> 8) as u8, (imm >> 16) as u8, (imm >> 24) as u8),
		4 => fail!(), // TODO: should mov and then add
		_ => unreachable!()
	}
}

#[inline(always)]
fn encode_subri(reg: super::JitReg, imm: u64) -> Vec<u8> {
	let rex = encode_rex(reg, None, true);
	match needed_bytes(imm) {
		1 => vec!(rex, 0x83, (0b11 << 6) + (0b101 << 3) + reg.to_real_reg(), imm as u8),
		2 | 3 => vec!(rex, 0x81, (0b11 << 6) + (0b101 << 3) + reg.to_real_reg(), imm as u8, (imm >> 8) as u8, (imm >> 16) as u8, (imm >> 24) as u8),
		4 => fail!(), // TODO: should mov and then add
		_ => unreachable!()
	}
}

#[inline(always)]
fn encode_mulrr(reg: super::JitReg, reg2: super::JitReg) -> Vec<u8> {
	let mut res = vec!();
	if reg != super::R1 { /* rax */
		res.push_all_move(encode_pushr(super::R1));
		res.push_all_move(encode_movrr(super::R1, reg));
	}
	if reg2 != super::R3 { /* rsi */
		res.push_all_move(encode_pushr(super::R3));
		res.push_all_move(encode_movrr(super::R3, reg2));
	}
	res.push_all([encode_rex(super::R3, None, true), 0xf7, (0b11 << 6) + (0b100 << 3) + super::R3.to_real_reg()]);
	if reg2 != super::R3 { /* rsi */
		res.push_all_move(encode_popr(super::R3));
	}
	if reg != super::R1 { /* rax */
		res.push_all_move(encode_movrr(reg, super::R1));
		res.push_all_move(encode_popr(super::R1));
	}
	res
}

#[inline(always)]
fn encode_mulri(reg: super::JitReg, imm: u64) -> Vec<u8> {
	let mut res = vec!();
	let immreg =
		if reg == super::R3 { /* rsi */
			super::R5 /* rcx */
		} else {
			super::R3
		};
	res.push_all_move(encode_pushr(immreg));
	res.push_all_move(encode_movri(immreg, imm));
	res.push_all_move(encode_mulrr(reg, immreg));
	res.push_all_move(encode_popr(immreg));
	res
}

#[inline(always)]
fn encode_pushr(reg: super::JitReg) -> Vec<u8> {
	let rex = encode_rex(reg, None, true);
	if rex == 0b01001000 {
		vec!(0x50 + reg.to_real_reg())
	} else {
		vec!(rex, 0x50 + reg.to_real_reg())
	}
}

#[inline(always)]
fn encode_popr(reg: super::JitReg) -> Vec<u8> {
	let rex = encode_rex(reg, None, true);
	if rex == 0b01001000 {
		vec!(0x58 + reg.to_real_reg())
	} else {
		vec!(rex, 0x58 + reg.to_real_reg())
	}
}

#[inline(always)]
fn encode_movrr(reg1: super::JitReg, reg2: super::JitReg) -> Vec<u8> {
	vec!(encode_rex(reg1, Some(reg2), true), 0x89, (0b11 << 6) + (reg2.to_real_reg() << 3) + reg1.to_real_reg() as u8)
}

#[inline(always)]
fn encode_movri(reg: super::JitReg, imm: u64) -> Vec<u8> {
	let rex = encode_rex(reg, None, false);
	match needed_bytes(imm) {
		1 | 2 | 3 => {
			if rex == 0b01000000 {
				vec!(0xb8 + reg.to_real_reg(), imm as u8, (imm >> 8) as u8, (imm >> 16) as u8, (imm >> 24) as u8)
			} else {
				vec!(rex, 0xb8 + reg.to_real_reg(), imm as u8, (imm >> 8) as u8, (imm >> 16) as u8, (imm >> 24) as u8)
			}
		}
		4 => vec!(rex + (1 << 3), 0xb8 + reg.to_real_reg(), imm as u8, (imm >> 8) as u8, (imm >> 16) as u8, (imm >> 24) as u8, (imm >> 32) as u8, (imm >> 40) as u8, (imm >> 48) as u8, (imm >> 56) as u8),
		_ => unreachable!()
	}
}

#[inline(always)]
fn encode_call<'a>(func: Option<&'a JitFunction<'a>>, pos: uint) -> Vec<u8> {
	match func {
		Some(func) => {
			let mut pos = func.label.pos as i32 - pos as i32;
			if pos > 0 {
				pos -= 5;
			}
			vec!(0xe8, pos as u8, (pos >> 8) as u8, (pos >> 16) as u8, (pos >> 24) as u8)
		}
		None => fail!() // XXX: fix
	}
}

#[inline(always)]
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

impl super::JitReg {
	pub fn to_real_reg(&self) -> u8 {
		match *self {
			super::R1  => 0b000, /* rax */
			super::R2  => 0b111, /* rdi */
			super::R3  => 0b110, /* rsi */
			super::R4  => 0b010, /* rdx */
			super::R5  => 0b001, /* rcx */
			super::R6  => 0b000, /* r8  */
			super::R7  => 0b001, /* r9  */
			super::R8  => 0b011, /* rbx */
			super::R9  => 0b010, /* r10 */
			super::R10 => 0b011, /* r11 */
			super::R11 => 0b100, /* r12 */
			super::R12 => 0b101, /* r13 */
			super::R13 => 0b110, /* r14 */
			super::R14 => 0b111, /* r15 */
			super::SP  => 0b100, /* rsp */
			super::BP  => 0b101  /* rbp */
		}
	}

	pub fn extended(&self) -> bool {
		match *self {
			super::R6 | super::R7 | super::R9 | super::R10 | super::R11 | super::R12 | super::R13 | super::R14 => true,
			_ => false
		}
	}
}

