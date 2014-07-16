/* This essentially shows three ways to use the JIT (macro asm, macro procedural, procedural with blocks) */

#![crate_type = "bin"]

#![feature(macro_rules, globs, phase)]

extern crate jit;
#[phase(plugin)] extern crate jit_macros;
extern crate libc;

use jit::*;

fn main() {
	let mut jit = jit::Jit::new();
	/*jit_asm!(jit,
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
	);*/
	jit_makefn!(jit, add_four, {
		add_four.push(Movrr(R1, R2));
		add_four.push(Addri(R1, 4));
		add_four.push(Call("sub_four"));
	});
	jit_makefn!(jit, sub_four, {
		sub_four.push(Movrr(R1, R2));
		sub_four.push(Subri(R1, 4));
		sub_four.push(Call("random_stuff"));
	});
	jit_makefn!(jit, random_stuff, {
		random_stuff.push(Subri(R1, 7));
		random_stuff.push(Addri(R1, 1000000000));
		random_stuff.push(Subri(R1, 500000000));
		random_stuff.push(Movri(R1, 1));
		random_stuff.push(Movri(R1, 10000));
		random_stuff.push(Movri(R1, 1000000000000));
		random_stuff.push(Movri(R1, 10));
		random_stuff.push(Mulri(R1, 10));
		random_stuff.push(Divri(R1, 100));
	});
	/*{
		let func = jit.function("add_four".to_string());
		func.push(Movrr(R1, R2));
		func.push(Addri(R1, 4));
		func.push(Call("sub_four"));
		func.end();
	}
	{
		let subfunc = jit.function("sub_four".to_string());
		subfunc.push(Movrr(R1, R2));
		subfunc.push(Subri(R1, 4));
		subfunc.push(Call("random_stuff"));
		subfunc.end();
	}
	{
		let randfunc = jit.function("random_stuff".to_string());
		randfunc.push(Subri(R1, 7));
		randfunc.push(Addri(R1, 1000000000));
		randfunc.push(Subri(R1, 500000000));
		randfunc.push(Movri(R1, 1));
		randfunc.push(Movri(R1, 10000));
		randfunc.push(Movri(R1, 1000000000000));
		randfunc.end();
	}*/

	let (region, add) = jit_compilefn!(jit, (int) -> int);
	println!("add(4): {}", add(4));
	drop(region);  // stops warning message
}

