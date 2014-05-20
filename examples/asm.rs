#![crate_type = "bin"]

#![feature(macro_rules, globs, phase)]

extern crate jit;
#[phase(syntax)] extern crate jit_macros;
extern crate libc;

use std::os;
use jit::{Movrr, Addi, Ret, R1, R2, Compilable};
use jit::region::MemoryRegion;

fn main() {
	let mut jit = jit::Jit::new();
	jit_asm!(jit,
		fn add_four:
			Movrr R1, R2;
			Addi R1, 4;
			Ret
	);
	let code = jit.compile();
	/*let func = jit.function("add_four".to_owned());
	func.push(Movrr(R1, R2));
	func.push(Addi(R1, 4));
	func.push(Ret);
	let code = func.compile();*/

	let mut region = match os::MemoryMap::new(code.len(), [os::MapReadable, os::MapWritable]) {
		Ok(m) => m,
		Err(f) => fail!(f.to_str())
	};

	type AddFourFn = extern "C" fn(int) -> int;
	let add = jit_func::<AddFourFn>(&mut region, code);
	println!("add(4): {}", add(4));
}

fn jit_func<T>(region: &mut os::MemoryMap, contents: &[u8]) -> T {
	region.copy(contents);
	region.protect();
	unsafe { std::mem::transmute(region.data) }
}
