//! Utility functions.

use hazardflow_macro::magic;

/// Returns ceiling log2.
pub const fn clog2(value: usize) -> usize {
    if value == 0 {
        0
    } else if value == 1 {
        1
    } else {
        (::core::mem::size_of::<usize>() * 8) - (value - 1).leading_zeros() as usize
    }
}

/// Returns minimum value.
pub const fn min(lhs: usize, rhs: usize) -> usize {
    if lhs < rhs {
        lhs
    } else {
        rhs
    }
}

/// Returns maximum value.
pub const fn max(lhs: usize, rhs: usize) -> usize {
    if lhs > rhs {
        lhs
    } else {
        rhs
    }
}

/// Display function
#[magic(system::display)]
pub fn display<V: Copy>(_fstring: &str, _args: V) {
    panic!("compiler magic")
}

/// Display macro
///
/// This macro will be compiled as `fdisplay` system task of verilog.
#[macro_export]
macro_rules! display {
    ($string: expr) => {
        $crate::std::utils::display($string, ())
    };
    ($fstring: expr, $($arg:expr),+) => {
        $crate::std::utils::display($fstring, ($($arg,)*))
    };
}

/// Assertion function
#[magic(system::assert)]
pub fn assert<V: Copy>(_cond: bool, _fstring: &str, _args: V) {
    panic!("compiler magic")
}

/// Assert macro
///
/// ## Syntax
///
/// \<assert> :=
///   hassert!(cond, string) | hassert!(cond, format_string, arg1, arg2, ...)
///
/// This macro will be compiled as below.
/// ```verilog
/// if (~cond {& current path condition}) begin
///    $fdisplay(format_string, arg1, arg2, ...);
///    $finish;
/// end
/// ```
///
/// ## Formatting
///
/// For the format string, use the syntax of verilog `$display` system task.
#[macro_export]
macro_rules! hassert {
    ($cond: expr, $string: expr) => {
        $crate::std::utils::assert($cond, $string, ())
    };
    ($cond: expr, $fstring: expr, $($arg:expr),+) => {
        $crate::std::utils::assert($cond, $fstring, ($($arg,)*))
    };
}

/// Panic macro
///
/// ## Syntax
///
/// \<panic> :=
///   hpanic!(cond, string) | hpanic!(cond, format_string, arg1, arg2, ...)
///
/// This macro will be compiled as below.
/// ```verilog
/// if (true {& current path condition}) begin
///    $fdisplay(format_string, arg1, arg2, ...);
///    $finish;
/// end
/// ```
///
/// ## Formatting
///
/// For the format string, use the syntax of verilog `$display` system task.
///
/// NOTE: Currently we do not support printing composite types like structs, tuples or arrays.
#[macro_export]
macro_rules! hpanic {
    ($fstring: expr, $($arg:expr),+) => {
        unsafe {
            $crate::std::utils::assert(false, $fstring, ($($arg,)*));
            $crate::std::value::x()
        }
    };
    ($string: expr) => {
        unsafe {
            $crate::std::utils::assert(false, $string, ());
            $crate::std::value::x()
        }
    };
}
