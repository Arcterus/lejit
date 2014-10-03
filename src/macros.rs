#![crate_name = "lejit_macros"]
#![crate_type = "rlib"]

#![feature(macro_rules)]

// TODO: probably just rewrite as a syntax extension
#[macro_export]
macro_rules! jit_asm (
   ($jit:ident, $(fn $flabel:ident : $($(. $sublabel:ident :)* $op:path $($operands:expr),*);*)+) => ({
      $({
         let func = $jit.function(stringify!($flabel).to_string());
         $(
            $(
               func.label(stringify!($sublabel).to_string());
            )*
            let func = func.op(op_asm!($op $(,$operands)*));
         )*
         func.end();
      })+
   })
)

#[macro_export]
macro_rules! op_asm (
   ($op:path, $($operands:expr),+) => (
      $op($($operands),+)
   );
   ($op:path) => (
      $op
   )
)

#[macro_export]
macro_rules! jit_compilefn (
   ($jit:ident, ($($types:ty),+) -> $rettype:ty) => ({
      type JitFnType = extern "C" fn($($types),+) -> $rettype;
      let region = $jit.region();
      let addr = region.data();
      (region, unsafe { ::std::mem::transmute::<*mut u8, JitFnType>(addr) })
   })
)
