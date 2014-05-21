#[cfg(target_arch = "x86_64")]
pub use self::x86_64::Jit;

pub mod x86_64;

pub trait Compilable<'a> {
	fn compile(&self, jit: &'a Jit<'a>, pos: uint) -> ~[u8];
}

pub enum JitOp<'a> {
	Addri(JitReg, u64),
	Subri(JitReg, u64),
	Movrr(JitReg, JitReg),
	Movri(JitReg, u64),
	Call(&'a str),
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

pub struct JitFunction<'a> {
	pub label: JitLabel,
	pub sublabels: Vec<JitLabel>,
	pub ops: Vec<JitOp<'a>>
}

pub struct JitLabel {
	name: ~str,
	pos: uint
}

impl<'a> JitFunction<'a> {
	pub fn new(name: ~str, pos: uint) -> JitFunction<'a> {
		JitFunction {
			label: JitLabel::new(name, pos),
			sublabels: vec!(),
			ops: vec!()
		}
	}

	pub fn push(&mut self, op: JitOp<'a>) {
		self.ops.push(op);
	}

	pub fn label(&mut self, name: ~str) {
		self.sublabels.push(JitLabel::new(name, self.ops.len()));
	}

	pub fn end(&mut self) {
		self.ops.push(Ret);
	}

	pub fn len(&self) -> uint {
		let mut len = 0;
		for op in self.ops.iter() {
			len += op.len();
		}
		len
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

