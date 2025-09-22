pub fn read_punct(s: &str, i: usize) -> usize {
    let puncts = ["<<=", ">>=", "...", "==", "!=", "<=", ">=", "->", "+=",
                  "-=", "*=", "/=", "++", "--", "%=", "&=", "|=", "^=",
                  "&&", "||", "<<", ">>", "##"];
    let rem = &s[i..];
    for p in puncts.iter() {
        if rem.starts_with(p) {
            return p.len();
        }
    }
    // Single-byte ASCII punct
    let bs = s.as_bytes();
    if let Some(&b) = bs.get(i) {
        if (33u8..=47u8).contains(&b) || (58u8..=64u8).contains(&b) || (91u8..=96u8).contains(&b) || (123u8..=126u8).contains(&b) {
            return 1;
        }
    }
    0
}
