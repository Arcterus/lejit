/* This essentially shows three ways to use the JIT (macro asm, macro procedural, procedural with blocks) */

#![crate_type = "bin"]

#![feature(macro_rules, globs, phase)]

extern crate lejit;
#[phase(plugin)] extern crate lejit_macros;
extern crate libc;

use lejit::*;

fn main() {
   let mut jit = Jit::new();
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

   jit.function("add_four".to_string())
      .op(Movrr(R1, R2))
      .op(Addri(R1, 4))
      .op(Call("sub_four"))
      .end();

   let subfunc = jit.function("sub_four".to_string());
   subfunc.op(Movrr(R1, R2))
          .op(Subri(R1, 4))
          .op(Call("random_stuff"))
          .end();

   let randfunc = jit.function("random_stuff".to_string());
   let op = randfunc.op(Subri(R1, 7))
                    .op(Addri(R1, 1000000000));
   let op2 = op.op(Subri(R1, 500000000))
               .op(Movri(R1, 1))
               .op(Movri(R1, 10000));
   op2.op(Movri(R1, 1000000000000))
      .end();

   let (region, add) = jit_compilefn!(jit, (int) -> int);
   println!("add(4): {}", add(4));
   drop(region);  // stops warning message
}
