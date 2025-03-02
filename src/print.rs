use core::str;

#[derive(Debug)]
pub struct ParseError;

#[allow(unused)]
pub fn slice_to_str((slice, len): (&[u8; 33], usize)) -> Result<&str, ParseError> {
    let real_part = &slice[65 - len..65];

    match str::from_utf8(real_part) {
        Ok(s) => Ok(s),
        Err(_) => Err(ParseError {}),
    }
}
#[allow(unused)]
pub fn u32_to_base(mut addr: u32, base: u8) -> Result<([u8; 33], usize), ()> {
    if !(2..=16).contains(&base) {
        return Err(());
    }

    let mut buf: [u8; 33] = [0; 33];
    let digits: &[u8; 16] = b"0123456789ABCDEF";

    if addr == 0 {
        buf[32] = b'0';
        return Ok((buf, 1));
    }

    let mut idx = buf.len();

    while addr != 0 && idx > 0 {
        idx -= 1;
        buf[idx] = digits[(addr % base as u32) as usize];
        addr /= base as u32;
    }

    if addr != 0 {
        return Err(());
    }

    let len = buf.len() - idx;

    Ok((buf, len))
}

#[cfg(test)]
mod u32_to_base_test {
    use super::*;

    #[test]
    fn test_normal_functionality_base_16_ff() {
        let num = 255u32;

        let res = match u32_to_base(num, 16) {
            Ok((len, buf)) => (len, buf),
            _ => ([0u8; 33], 0),
        };

        let result_slice = &res.0[33 - res.1..];

        let result_str = core::str::from_utf8(result_slice).unwrap();

        assert_eq!(result_str, "FF");
    }

    #[test]
    fn test_normal_functionality_base_16_ffff() {
        let num = 65535u32;

        let res = match u32_to_base(num, 16) {
            Ok((len, buf)) => (len, buf),
            _ => ([0u8; 33], 0),
        };

        let result_slice = &res.0[33 - res.1..];

        let result_str = core::str::from_utf8(result_slice).unwrap();

        assert_eq!(result_str, "FFFF");
    }

    #[test]
    fn test_normal_functionality_base_16_ffffff() {
        let num = 16777215u32;

        let res = match u32_to_base(num, 16) {
            Ok((len, buf)) => (len, buf),
            _ => ([0u8; 33], 0),
        };

        let result_slice = &res.0[33 - res.1..];

        let result_str = core::str::from_utf8(result_slice).unwrap();

        assert_eq!(result_str, "FFFFFF");
    }

    #[test]
    fn test_normal_functionality_base_16_ffffffff() {
        let num = 4294967295u32;

        let res = match u32_to_base(num, 16) {
            Ok((len, buf)) => (len, buf),
            _ => ([0u8; 33], 0),
        };

        let result_slice = &res.0[33 - res.1..];

        let result_str = core::str::from_utf8(result_slice).unwrap();

        assert_eq!(result_str, "FFFFFFFF");
    }
}
