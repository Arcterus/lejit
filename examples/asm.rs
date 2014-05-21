#![crate_type = "bin"]

#![feature(macro_rules, globs, phase)]

extern crate jit;
#[phase(syntax)] extern crate jit_macros;
extern crate libc;

use jit::{Movrr, Addri, Subri, Movri, Call, R1, R2};

fn main() {
	let mut jit = jit::Jit::new();
	jit_asm!(jit,
		fn add_four:
			Movrr R1, R2;
			Addri R1, 4;
			Call "sub_four"
		fn sub_four:
			Movrr R1, R2;
			Subri R1, 4;
			Call "random_stuff"
		fn random_stuff:
			Subri R1, 7;
			Addri R1, 1000000000;
			Subri R1, 500000000;
			Movri R1, 1;
			Movri R1, 10000;
			Movri R1, 1000000000000
	);
	//let code = jit.compile();
	/*let func = jit.function("add_four".to_owned());
	func.push(Movrr(R1, R2));
	func.push(Addri(R1, 4));
	func.push(Call("sub_four"));
	func.end();
	let subfunc = jit.function("sub_four".to_owned());
	subfunc.push(Movrr(R1, R2));
	subfunc.push(Subri(R1, 4));
	subfunc.push(Call("random_stuff"));
	subfunc.end();
	let randfunc = jit.function("random_stuff".to_owned());
	randfunc.push(Subri(R1, 7));
	randfunc.push(Addri(R1, 1000000000));
	randfunc.push(Subri(R1, 500000000));
	randfunc.push(Movri(R1, 1));
	randfunc.push(Movri(R1, 10000));
	randfunc.push(Movri(R1, 1000000000000));
	randfunc.end();
	let code = jit.compile();*/

	let (region, add) = jit_makefn!(jit, (int) -> int);
	println!("add(4): {}", add(4));
}

/*fn jit_func<T>(region: &mut os::MemoryMap, contents: &[u8]) -> T {
	unsafe { std::mem::transmute(region.data) }
}*/
