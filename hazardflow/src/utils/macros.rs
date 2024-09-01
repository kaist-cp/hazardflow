//! Macros.

// /// Log while synthesizing verilog code.
// #[macro_export]
// macro_rules! log {
//     (DEBUG, $($arg:tt)*) => {
//         log::debug!("{} (L{})", format_args!($($arg)*), line!())
//     };
//     (INFO, $($arg:tt)*) => {
//         log::info!("{}", format_args!($($arg)*))
//     };
// }

/// Ok or executing the given expression.
#[macro_export]
macro_rules! ok_or {
    ($e:expr, $err:expr) => {{
        match $e {
            Ok(r) => r,
            Err(_) => $err,
        }
    }};
}

/// Some or executing the given expression.
#[macro_export]
macro_rules! some_or {
    ($e:expr, $err:expr) => {{
        match $e {
            Some(r) => r,
            None => $err,
        }
    }};
}
