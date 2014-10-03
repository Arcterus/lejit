use std::os;
use region::MemoryRegion;

#[cfg(target_arch = "x86_64")]
#[path = "x86_64.rs"]
pub mod backend;

pub trait Compilable<'a> {
   fn compile(&self, jit: &'a Jit<'a>, pos: uint) -> Vec<u8>;
}

pub trait Opcode {
   fn len(&self) -> uint;
}

pub trait Register {
   fn to_real_reg(&self) -> u8;
   fn extended(&self) -> bool;
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

#[deriving(PartialEq)]
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
   pub ops: Vec<JitOp<'a>>,
   jit: Option<*mut Jit<'a>>,   // TODO: try to convert to borrowed pointer
   len: uint
}

pub struct JitLabel {
   name: String,
   pos: uint
}

pub struct JitOpcode<'a> {
   func: JitFunction<'a>,
   op: JitOp<'a>
}

impl<'a> Jit<'a> {
   pub fn new() -> Jit<'a> {
      Jit {
         funcs: vec!()
      }
   }

   pub fn function<'x>(&'x mut self, name: String) -> JitFunction<'a> {
      let len = self.funcs.len();
      let pos =
         if self.funcs.is_empty() {
            0
         } else {
            let oldfn = &self.funcs[len - 1];
            oldfn.label.pos + oldfn.len()
         };
      let jit: *mut Jit<'a> = self;
      JitFunction::new(name, Some(jit), pos)
   }

   pub fn find_function<'x>(&'a self, name: &str) -> Option<&'x JitFunction<'a>> {
      // TODO: redesign so don't have to iterate through an array
      for func in self.funcs.iter() {
         let fname: &str = func.label.name.as_slice();
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
         vec.extend(comp.into_iter());
      }
      vec
   }

   pub fn region(&'a self) -> os::MemoryMap {
      let code = self.compile();
      let mut region = match os::MemoryMap::new(code.len(), [os::MapReadable, os::MapWritable]) {
         Ok(m) => m,
         Err(f) => fail!(f)
      };
      region.copy(code.as_slice());
      region.protect();
      region
   }
}

impl<'a> JitFunction<'a> {
   pub fn new(name: String, jit: Option<*mut Jit<'a>>, pos: uint) -> JitFunction<'a> {
      JitFunction {
         label: JitLabel::new(name, pos),
         sublabels: vec!(),
         ops: vec!(),
         jit: jit,
         len: 0
      }
   }

   pub fn op(mut self, op: JitOp<'a>) -> JitOpcode<'a> {
      self.len += op.len();
      self.ops.push(op);
      JitOpcode::new(self, op)
   }

   pub fn label(&mut self, name: String) {
      self.sublabels.push(JitLabel::new(name, self.ops.len()));
   }

   pub fn end(mut self) {
      self.len += (Ret).len();
      self.ops.push(Ret);
      let jit = self.jit.unwrap();
      self.jit = None;
      unsafe { (*jit).funcs.push(self) };
   }

   pub fn len(&self) -> uint { self.len }
}

impl JitLabel {
   pub fn new(name: String, pos: uint) -> JitLabel {
      JitLabel {
         name: name,
         pos: pos
      }
   }
}

impl<'a> JitOpcode<'a> {
   pub fn new(func: JitFunction<'a>, op: JitOp<'a>) -> JitOpcode<'a> {
      JitOpcode {
         func: func,
         op: op
      }
   }

   pub fn op(self, op: JitOp<'a>) -> JitOpcode<'a> {
      self.func.op(op)
   }

   pub fn end(self) {
      self.func.end()
   }
}

impl<'a> Opcode for JitOpcode<'a> {
   fn len(&self) -> uint { self.op.len() }
}
