pub fn decode_string(bs: &[u8], mut i: usize) -> (String, usize) {
    let mut buf = String::new();
    while i < bs.len() && bs[i] != b'"' {
        if bs[i] == b'\\' {
            i += 1;
            if i >= bs.len() { break; }
            let c = bs[i];
            // Octal escape: up to three octal digits
            if c >= b'0' && c <= b'7' {
                let mut val = (c - b'0') as u32;
                i += 1;
                for _ in 0..2 {
                    if i < bs.len() {
                        let d = bs[i];
                        if d >= b'0' && d <= b'7' {
                            val = (val << 3) + (d - b'0') as u32;
                            i += 1;
                            continue;
                        }
                    }
                    break;
                }
                buf.push(char::from_u32(val).unwrap_or('\u{FFFD}'));
                continue;
            }
            // Hex escape: \x followed by hex digits (any number)
            if c == b'x' {
                i += 1;
                let mut val: u32 = 0;
                let mut digits = 0usize;
                while i < bs.len() {
                    let d = bs[i];
                    let digit = if d >= b'0' && d <= b'9' { Some((d - b'0') as u32) }
                                else if d >= b'a' && d <= b'f' { Some((d - b'a' + 10) as u32) }
                                else if d >= b'A' && d <= b'F' { Some((d - b'A' + 10) as u32) }
                                else { None };
                    if let Some(v) = digit {
                        val = (val << 4) + v;
                        i += 1;
                        digits += 1;
                        continue;
                    }
                    break;
                }
                if digits > 0 {
                    buf.push(char::from_u32(val).unwrap_or('\u{FFFD}'));
                }
                continue;
            }

            // Single-character escapes
            let ch = match c {
                b'n' => '\n',
                b'r' => '\r',
                b't' => '\t',
                b'0' => '\0',
                b'\\' => '\\',
                b'"' => '"',
                b'\'' => '\'',
                b'a' => '\u{0007}',
                b'b' => '\u{0008}',
                b'v' => '\u{000B}',
                b'f' => '\u{000C}',
                b'e' => '\u{001B}', // GNU extension
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
    // Similar handling as decode_string for escapes
    let mut val: u32 = 0;
    if i < bs.len() && bs[i] == b'\\' {
        i += 1;
        if i >= bs.len() { return (0, i); }
        let c = bs[i];
        if c >= b'0' && c <= b'7' {
            val = (c - b'0') as u32;
            i += 1;
            for _ in 0..2 {
                if i < bs.len() {
                    let d = bs[i];
                    if d >= b'0' && d <= b'7' {
                        val = (val << 3) + (d - b'0') as u32;
                        i += 1;
                        continue;
                    }
                }
                break;
            }
            return (val as i64, i);
        }
        if c == b'x' {
            i += 1;
            let mut v: u32 = 0;
            let mut digits = 0usize;
            while i < bs.len() {
                let d = bs[i];
                let dig = if d >= b'0' && d <= b'9' { Some((d - b'0') as u32) }
                          else if d >= b'a' && d <= b'f' { Some((d - b'a' + 10) as u32) }
                          else if d >= b'A' && d <= b'F' { Some((d - b'A' + 10) as u32) }
                          else { None };
                if let Some(dd) = dig { v = (v << 4) + dd; i += 1; digits += 1; continue; }
                break;
            }
            // If no hex digits were found, return 0 as fallback.
            if digits > 0 { return (v as i64, i); }
            return (0, i);
        }
        let ch = match c {
            b'n' => '\n' as u32,
            b'r' => '\r' as u32,
            b't' => '\t' as u32,
            b'0' => '\0' as u32,
            b'\\' => '\\' as u32,
            b'\'' => '\'' as u32,
            b'a' => 0x07,
            b'b' => 0x08,
            b'v' => 0x0B,
            b'f' => 0x0C,
            b'e' => 0x1B,
            other => other as u32,
        };
        i += 1;
        return (ch as i64, i);
    }
    if i < bs.len() { val = bs[i] as u32; i += 1; }
    (val as i64, i)
}
