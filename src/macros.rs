#[macro_export]
macro_rules! retry_until_ok {
    ($condition:expr) => {
        loop {
            if let Ok(ok) = $condition {
                break ok;
            }
            core::hint::spin_loop();
        }
    };
}

/// Our panic handler currently cannot handle the messages from `.expect()`,
/// resulting in a non-informative panic message.
#[macro_export]
macro_rules! expect_opt {
    ($x:expr, $msg:literal) => {{
        let Some(x) = $x else {
            panic!(stringify!($msg));
        };
        x
    }};
}

#[macro_export]
macro_rules! expect_res {
    ($x:expr, $msg:literal) => {{
        let Ok(x) = $x else {
            panic!(stringify!($msg));
        };
        x
    }};
}

#[macro_export]
macro_rules! kassert_eq {
    ($a:expr, $b:expr, $c:expr) => {
        if $a != $b {
            return Err(stringify!($c));
        }
    };
    ($a:expr, $b:expr) => {
        if $a != $b {
            return Err(concat!("Assertion failed: ", stringify!($a), " != ", stringify!($b)));
        }
    };
}

#[macro_export]
macro_rules! kassert {
    ($a:expr, $b:expr) => {
        if !($a) {
            return Err(stringify!($b));
        }
    };
    ($a:expr) => {
        if !($a) {
            return Err(concat!("Assertion failed: ", stringify!($a)));
        }
    };
}
