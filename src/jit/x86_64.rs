use std::vec::FromVec;
use super::{Compilable, JitFunction, JitLabel, JitOp, JitReg};

pub static REXW: u8 = 0x48;
pub static ADDRI: [u8, ..2] = [REXW, 0x83];
pub static MOVRR: [u8, ..2] = [REXW, 0x89];
pub static RET: u8 = 0xc3;

pub struct Jit<'a> {
	funcs: Vec<JitFunction>
}

impl<'a> Jit<'a> {
	pub fn new() -> Jit<'a> {
		Jit {
			funcs: vec!()
		}
	}

	pub fn function(&'a mut self, name: ~str) -> &'a mut JitFunction {
		let len = self.funcs.len();
		let func = JitFunction::new(name, if self.funcs.is_empty() {
			0
		} else {
			let oldfn = self.funcs.get(len - 1);
			oldfn.label.pos + oldfn.ops.len()
		});
		self.funcs.push(func);
		self.funcs.get_mut(len)
	}
}

impl<'a> Compilable for Jit<'a> {
	fn compile(&self) -> ~[u8] {
		let mut vec = vec!();
		for func in self.funcs.iter() {
			vec.push_all(func.compile());
		}
		FromVec::from_vec(vec)
	}
}

impl Compilable for JitFunction {
	fn compile(&self) -> ~[u8] {
		let mut vec = vec!();
		for op in self.ops.iter() {
			let comp = op.compile();
			println!("{}", comp);
			vec.push_all(comp);
		}
		FromVec::from_vec(vec)
	}
}

impl Compilable for JitOp {
	fn compile(&self) -> ~[u8] {
		match *self {
			super::Addi(reg, imm) => FromVec::from_vec(Vec::from_slice(ADDRI).append([(0b11 << 6) + reg.to_real_reg(), imm as u8])),
			super::Movrr(reg1, reg2) => FromVec::from_vec(Vec::from_slice(MOVRR).append([(0b11 << 6) + (reg2.to_real_reg() << 3) + reg1.to_real_reg() as u8])),
			super::Ret => ~[RET],
			_ => fail!() // TODO: implement all ops
		}
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

//unsafe fn jit_func<T>()
