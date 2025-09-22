pub fn read_number(s: &str, i: usize) -> usize {
    let bs = s.as_bytes();
    let n = bs.len();
    if i >= n || !(b'0' <= bs[i] && bs[i] <= b'9') {
        return 0;
    }
    let mut j = i;
    while j < n && (b'0' <= bs[j] && bs[j] <= b'9') {
        j += 1;
    }
    j - i
}
