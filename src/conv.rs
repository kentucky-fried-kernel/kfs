/// Converts a slice of bytes into a `usize`, assuming hexadecimal format, skipping leading and
/// trailing whitespaces.
///
/// Returns `None` if `bytes` cannot be converted to a `usize` deterministically.
pub fn hextou(bytes: &[u8]) -> Option<usize> {
    let mut starting_idx = 0;
    while b"\t \n".contains(&bytes[starting_idx]) {
        starting_idx += 1;
    }
    let num: &[u8] = if bytes[starting_idx] == b'0' && bytes[starting_idx + 1] == b'x' {
        &bytes[starting_idx + 2..]
    } else {
        &bytes[starting_idx..]
    };

    let mut result: usize = 0;

    for byte in num {
        let digit: u8;

        if *byte >= b'0' && *byte <= b'9' {
            digit = *byte - b'0';
        } else if *byte >= b'a' && *byte <= b'f' {
            digit = *byte - b'a' + 10;
        } else if *byte >= b'A' && *byte <= b'F' {
            digit = *byte - b'A' + 10;
        } else if b"\t \n\0".contains(byte) {
            break;
        } else {
            return None;
        }

        result = result * 16 + digit as usize;
    }
    Some(result)
}
