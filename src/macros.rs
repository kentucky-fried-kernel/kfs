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
