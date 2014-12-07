/*
 * Copyright (c) 2014 Arcterus
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#![crate_name = "lejit"]

#![crate_type = "dylib"]
#![crate_type = "rlib"]

#![feature(globs, macro_rules)]

#![experimental]

//! A (hopefully) simple JIT library.
//!
//! # Example
//!
//! ```
//! #![feature(macro_rules, globs, phase)]
//!
//! #[phase(plugin, link)]
//! extern crate lejit;
//! extern crate libc;
//!
//! use lejit::*;
//! use lejit::JitOp::*;
//! use lejit::JitReg::*;
//!
//! fn main() {
//!    let mut jit = Jit::new();
//!
//!    jit.function("add_four".to_string())
//!       .op(Movrr(R1, R2))
//!       .op(Addri(R1, 4))
//!       .op(Call("sub_four"))
//!       .end();
//!
//!    let subfunc = jit.function("sub_four".to_string());
//!    subfunc.op(Movrr(R1, R2))
//!           .op(Subri(R1, 4))
//!           .op(Call("random_stuff"))
//!           .end();
//!
//!    let randfunc = jit.function("random_stuff".to_string());
//!    let op = randfunc.op(Subri(R1, 7))
//!                     .op(Addri(R1, 1000000000));
//!    let op2 = op.op(Subri(R1, 500000000))
//!                .op(Movri(R1, 1))
//!                .op(Movri(R1, 10000));
//!    op2.op(Movri(R1, 1000000000000))
//!       .end();
//!
//!    let add = jit_compilefn!(jit, (int) -> int);
//!    println!("add(4): {}", add(4));
//! }
//! ```

extern crate libc;

pub use jit::*;

pub mod region;
pub mod jit;

#[macro_export]
macro_rules! jit_compilefn (
   ($jit:ident, ($($types:ty),+) -> $rettype:ty) => ({
      type JitFnType = extern "C" fn($($types),+) -> $rettype;
      let region = $jit.region();
      let addr = region.data();
      unsafe { ::std::mem::transmute::<*mut u8, JitFnType>(addr) }
   })
)
