pub fn decode_string(bs: &[u8], mut i: usize) -> (String, usize) {
    let mut buf = String::new();
    while i < bs.len() && bs[i] != b'"' {
        if bs[i] == b'\\' {
            i += 1;
            if i >= bs.len() { break; }
            let ch = match bs[i] {
                b'n' => '\n',
                b'r' => '\r',
                b't' => '\t',
                b'0' => '\0',
                b'\\' => '\\',
                b'"' => '"',
                b'\'' => '\'',
                other => other as char,
            };
            buf.push(ch);
            i += 1;
            continue;
        }
        buf.push(bs[i] as char);
        i += 1;
    }
    (buf, i)
}

pub fn decode_char(bs: &[u8], mut i: usize) -> (i64, usize) {
    let mut val = 0i64;
    if i < bs.len() && bs[i] == b'\\' {
        i += 1;
        if i < bs.len() { val = bs[i] as i64; i += 1; }
    } else if i < bs.len() {
        val = bs[i] as i64; i += 1;
    }
    (val, i)
}
