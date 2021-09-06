pub fn left_pad(s: &[u8], length: usize) -> Vec<u8> {
    let mut buf = vec![0u8; length - s.len()];
    buf.extend_from_slice(&s[0..s.len()]);
    buf
}
