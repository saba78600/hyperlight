use std::usize;

pub fn is_ident_start_byte(b: u8) -> bool {
    b == b'_' || (b'a' <= b && b <= b'z') || (b'A' <= b && b <= b'Z')
}

pub fn is_ident_continue_byte(b: u8) -> bool {
    is_ident_start_byte(b) || (b'0' <= b && b <= b'9')
}

pub fn read_ident(s: &str, i: usize) -> usize {
    let bs = s.as_bytes();
    let n = bs.len();
    if i >= n || !is_ident_start_byte(bs[i]) {
        return 0;
    }
    let mut j = i + 1;
    while j < n && is_ident_continue_byte(bs[j]) {
        j += 1;
    }
    j - i
}
