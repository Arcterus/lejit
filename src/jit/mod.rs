#![experimental]

use std::collections::BTreeMap;
use std::os;
use region::MemoryRegion;

#[cfg(target_arch = "x86_64")]
#[path = "x86_64.rs"]
mod backend;

#[experimental]
pub trait Compilable<'a, 'b> {
   fn compile(&self, jit: &'a Jit<'a, 'b>, pos: uint) -> Vec<u8>;
}

#[experimental]
pub trait Opcode {
   fn len(&self) -> uint;
}

#[experimental]
pub trait Register {
   fn to_real_reg(&self) -> u8;
   fn extended(&self) -> bool;
}

#[experimental]
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
#[experimental]
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

#[experimental]
pub struct Jit<'a:'b, 'b> {
   funcs: BTreeMap<String, JitFunction<'a, 'b>>,
   last_func: Option<&'b JitFunction<'a, 'b>>,
   region: Option<os::MemoryMap>
}

#[experimental]
pub struct JitFunction<'a:'b, 'b> {
   pub label: JitLabel,
   pub sublabels: Vec<JitLabel>,
   pub ops: Vec<JitOp<'a>>,
   jit: Option<&'b mut Jit<'a, 'b>>,
   len: uint
}

#[experimental]
pub struct JitLabel {
   name: String,
   pos: uint
}

#[experimental]
pub struct JitOpcode<'a:'b, 'b> {
   func: JitFunction<'a, 'b>,
   op: JitOp<'a>
}

#[experimental]
impl<'a, 'b> Jit<'a, 'b> {
   pub fn new() -> Jit<'a, 'b> {
      Jit {
         funcs: BTreeMap::new(),
         last_func: None,
         region: None
      }
   }

   /// Creates a function with the given name and returns a JitFunction for the
   /// new function.
   fn function(&'b mut self, name: String) -> JitFunction<'a, 'b> {
      let pos = match self.last_func {
         Some(func) => func.label.pos + func.len(),
         None => 0
      };
      JitFunction::new(name, Some(self), pos)
   }

   /// Builds a function with the given name and using the given closure to add
   /// operations to the function.
   ///
   /// # Example
   ///
   /// ```
   /// let mut jit = Jit::new();
   /// jit.build_function("example".to_string(), |func| {
   ///    func.op(Movri(R1, 3)).end()
   /// });
   /// ```
   pub fn build_function(&'b mut self, name: String, cb: |func: JitFunction<'a, 'b>|) {
      let func = self.function(name);
      cb(func)
   }

   /// Tries to find and return the function with the given name.  If there is
   /// no function with the given name, None will be returned.
   pub fn find_function(&'a self, name: &str) -> Option<&'b JitFunction<'a, 'b>> {
      self.funcs.get(name)
   }

   /// Compiles the code that has been given to the JIT so far and returns the
   /// executable instructions.
   pub fn compile(&'a self) -> Vec<u8> {
      let mut vec = vec!();
      let mut pos = 0;
      for func in self.funcs.values() {
         let comp = func.compile(self, pos);
         pos += comp.len();
         vec.extend(comp.into_iter());
      }
      vec
   }

   /// Generates a memory mapped region for the executable code to be placed.
   /// The returned region will be invalidated if this function is called again.
   pub fn region(&'a mut self) -> &mut os::MemoryMap {
      let code = self.compile();
      let mut region = match os::MemoryMap::new(code.len(), &[os::MapReadable, os::MapWritable]) {
         Ok(m) => m,
         Err(f) => panic!(f)
      };
      region.copy(code.as_slice());
      region.protect();
      self.region = Some(region);
      self.region.as_mut().unwrap()
   }
}

#[experimental]
impl<'a:'b, 'b> JitFunction<'a, 'b> {
   pub fn new(name: String, jit: Option<&'b mut Jit<'a, 'b>>, pos: uint) -> JitFunction<'a, 'b> {
      JitFunction {
         label: JitLabel::new(name, pos),
         sublabels: vec!(),
         ops: vec!(),
         jit: jit,
         len: 0
      }
   }

   pub fn op(mut self, op: JitOp<'a>) -> JitOpcode<'a, 'b> {
      self.len += op.len();
      self.ops.push(op);
      JitOpcode::new(self, op)
   }

   pub fn label(&mut self, name: String) {
      self.sublabels.push(JitLabel::new(name, self.ops.len()));
   }

   pub fn end(mut self) {
      self.len += (JitOp::Ret).len();
      self.ops.push(JitOp::Ret);
      let jit = self.jit.unwrap();
      self.jit = None;
      let name = self.label.name.clone();
      let name_clone = name.clone();
      jit.funcs.insert(name, self);
      jit.last_func = jit.funcs.get(&name_clone);
   }

   pub fn len(&self) -> uint { self.len }
}

#[experimental]
impl JitLabel {
   pub fn new(name: String, pos: uint) -> JitLabel {
      JitLabel {
         name: name,
         pos: pos
      }
   }
}

#[experimental]
impl<'a:'b, 'b> JitOpcode<'a, 'b> {
   pub fn new(func: JitFunction<'a, 'b>, op: JitOp<'a>) -> JitOpcode<'a, 'b> {
      JitOpcode {
         func: func,
         op: op
      }
   }

   pub fn op(self, op: JitOp<'a>) -> JitOpcode<'a, 'b> {
      self.func.op(op)
   }

   pub fn end(self) {
      self.func.end()
   }
}

#[experimental]
impl<'a, 'b> Opcode for JitOpcode<'a, 'b> {
   fn len(&self) -> uint { self.op.len() }
}
