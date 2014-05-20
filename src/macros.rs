#![crate_id = "jit_macros#0.1-pre"]
#![crate_type = "rlib"]

#![feature(macro_rules)]

#![macro_escape]

#[macro_export]
macro_rules! jit_asm (
	($jit:ident, $(fn $flabel:ident : $($(. $sublabel:ident :)* $op:path $($operands:expr),*);*)+) => ({
		$(
			let func = $jit.function(stringify!($flabel).to_owned());
			$(
				$(
					func.label(stringify!($sublabel).to_owned());
				)*
				func.push(op_asm!($op $(,$operands)*));
			)*
		)+
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
