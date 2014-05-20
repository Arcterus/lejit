#[cfg(target_arch = "x86_64")]
pub use self::x86_64::Jit;

pub mod x86_64;

pub trait Compilable {
	fn compile(&self) -> ~[u8];
}

pub enum JitOp {
	Addi(JitReg, uint),
	Subi(JitReg, uint),
	Movrr(JitReg, JitReg),
	Ret
}

pub enum JitReg {
	R1,
	R2,
	R3,
	R4,
	R5,
	R6,
	R7,
	R8,
	R9,
	R10,
	R11,
	R12
}

pub struct JitFunction {
	pub label: JitLabel,
	pub sublabels: Vec<JitLabel>,
	pub ops: Vec<JitOp>
}

pub struct JitLabel {
	name: ~str,
	pos: uint
}

impl JitFunction {
	pub fn new(name: ~str, pos: uint) -> JitFunction {
		JitFunction {
			label: JitLabel::new(name, pos),
			sublabels: vec!(),
			ops: vec!()
		}
	}

	pub fn push(&mut self, op: JitOp) {
		self.ops.push(op);
	}

	pub fn label(&mut self, name: ~str) {
		self.sublabels.push(JitLabel::new(name, self.ops.len()));
	}

	pub fn end(&mut self) {
		self.ops.push(Ret);
	}
}

impl JitLabel {
	pub fn new(name: ~str, pos: uint) -> JitLabel {
		JitLabel {
			name: name,
			pos: pos
		}
	}
}

