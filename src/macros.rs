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
