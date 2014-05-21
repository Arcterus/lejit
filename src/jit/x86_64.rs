use std::{u8, u16, u32, u64};
use std::vec::FromVec;
use super::{Compilable, Jit, JitFunction, JitLabel, JitOp, JitReg};

//static REXW: u8 = 0x48;
static RET: u8 = 0xc3;

impl<'a> Compilable<'a> for JitFunction<'a> {
	fn compile(&self, jit: &'a Jit<'a>, pos: uint) -> ~[u8] {
		let mut vec = vec!();
		let mut pos = pos;
		for op in self.ops.iter() {
			let comp = op.compile(jit, pos);
			println!("{}", comp);
			pos += comp.len();
			vec.push_all(comp);
		}
		FromVec::from_vec(vec)
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
			super::Movrr(_, _) => 3,
			super::Movri(_, imm) => match imm {
				1 | 2 | 3 => 5,
				_ => 9
			},
			super::Call(_) => 5,
			super::Ret => 1
		}
	}
}

impl<'a> Compilable<'a> for JitOp<'a> {
	fn compile(&self, jit: &'a Jit<'a>, pos: uint) -> ~[u8] {
		match *self {
			super::Addri(reg, imm) => encode_addri(reg, imm),
			super::Subri(reg, imm) => encode_subri(reg, imm),
			super::Movrr(reg1, reg2) => ~[0x48, 0x89, (0b11 << 6) + (reg2.to_real_reg() << 3) + reg1.to_real_reg() as u8],
			super::Movri(reg, imm) => encode_movri(reg, imm),
			super::Call(name) => encode_call(jit.find_function(name), pos),
			super::Ret => ~[RET],
			//_ => fail!() // TODO: implement all ops
		}
	}
}

#[inline(always)]
fn encode_addri(reg: super::JitReg, imm: u64) -> ~[u8] {
	match needed_bytes(imm) {
		1 => ~[0x48, 0x83, (0b11 << 6) + reg.to_real_reg(), imm as u8],
		2 | 3 => ~[0x48, 0x81, (0b11 << 6) + reg.to_real_reg(), imm as u8, (imm >> 8) as u8, (imm >> 16) as u8, (imm >> 24) as u8],
		4 => fail!(), // TODO: should mov and then add
		_ => unreachable!()
	}
}

#[inline(always)]
fn encode_subri(reg: super::JitReg, imm: u64) -> ~[u8] {
	match needed_bytes(imm) {
		1 => ~[0x48, 0x83, (0b11 << 6) + (0b101 << 3) + reg.to_real_reg(), imm as u8],
		2 | 3 => ~[0x48, 0x81, (0b11 << 6) + (0b101 << 3) + reg.to_real_reg(), imm as u8, (imm >> 8) as u8, (imm >> 16) as u8, (imm >> 24) as u8],
		4 => fail!(), // TODO: should mov and then add
		_ => unreachable!()
	}
}

#[inline(always)]
fn encode_movri(reg: super::JitReg, imm: u64) -> ~[u8] {
	match needed_bytes(imm) {
		1 | 2 | 3 => ~[0xb8 + reg.to_real_reg(), imm as u8, (imm >> 8) as u8, (imm >> 16) as u8, (imm >> 24) as u8],
		4 => ~[0x48, 0xb8 + reg.to_real_reg(), imm as u8, (imm >> 8) as u8, (imm >> 16) as u8, (imm >> 24) as u8, (imm >> 32) as u8, (imm >> 40) as u8, (imm >> 48) as u8, (imm >> 56) as u8],
		_ => unreachable!()
	}
}

#[inline(always)]
fn encode_call<'a>(func: Option<&'a JitFunction<'a>>, pos: uint) -> ~[u8] {
	match func {
		Some(func) => {
			let mut pos = func.label.pos as i32 - pos as i32;
			if pos > 0 {
				pos -= 5;
			}
			// 2's complement
			~[0xe8, pos as u8, (pos >> 8) as u8, (pos >> 16) as u8, (pos >> 24) as u8]
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
			super::R1 => 0b000, /* rax */
			super::R2 => 0b111, /* rdi */ // TODO: change this from R2 to something else
			_ => fail!() // TODO: implement all registers
		}
	}
}

