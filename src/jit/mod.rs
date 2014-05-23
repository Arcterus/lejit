use std::os;
use region::MemoryRegion;

#[cfg(target_arch = "x86_64")]
#[path = "x86_64.rs"]
pub mod backend;

pub trait Compilable<'a> {
	fn compile(&self, jit: &'a Jit<'a>, pos: uint) -> Vec<u8>;
}

pub enum JitOp<'a> {
	Addri(JitReg, u64),
	Subri(JitReg, u64),
	Mulri(JitReg, u64),
	Mulrr(JitReg, JitReg),
	Divri(JitReg, u64),
	Divrr(JitReg, JitReg),
	Movrr(JitReg, JitReg),
	Movri(JitReg, u64),
	Pushr(JitReg),
	Popr(JitReg),
	Call(&'a str),
	Ret
}

#[deriving(Eq)]
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
	R12,
	R13,
	R14,
	SP,
	BP
}

pub struct Jit<'a> {
	funcs: Vec<JitFunction<'a>>
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

impl<'a> Jit<'a> {
	pub fn new() -> Jit<'a> {
		Jit {
			funcs: vec!()
		}
	}

	pub fn function<'x>(&'x mut self, name: ~str) -> &'x mut JitFunction<'a> {
		let len = self.funcs.len();
		let func = JitFunction::new(name, if self.funcs.is_empty() {
			0
		} else {
			let oldfn = self.funcs.get(len - 1);
			oldfn.label.pos + oldfn.len()
		});
		self.funcs.push(func);
		self.funcs.get_mut(len)
	}

	pub fn find_function(&'a self, name: &str) -> Option<&'a JitFunction<'a>> {
		// TODO: redesign so don't have to iterate through an array
		for func in self.funcs.iter() {
			let fname: &str = func.label.name;
			if fname == name {
				return Some(func);
			}
		}
		None
	}

	pub fn compile(&'a self) -> Vec<u8> {
		let mut vec = vec!();
		let mut pos = 0;
		for func in self.funcs.iter() {
			let comp = func.compile(self, pos);
			pos += comp.len();
			vec.push_all_move(comp);
		}
		vec
	}

	pub fn region(&'a self) -> os::MemoryMap {
		let code = self.compile();
		let mut region = match os::MemoryMap::new(code.len(), [os::MapReadable, os::MapWritable]) {
			Ok(m) => m,
			Err(f) => fail!(f.to_str())
		};
		region.copy(code.as_slice());
		region.protect();
		region
	}
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

