#![crate_type = "bin"]

#![feature(macro_rules, globs, phase)]

#[phase(plugin, link)]
extern crate lejit;
extern crate libc;

use lejit::*;
use lejit::JitOp::*;
use lejit::JitReg::*;

fn main() {
   let mut jit = Jit::new();

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

   let add = jit_compilefn!(jit, (int) -> int);
   println!("add(4): {}", add(4));
}
