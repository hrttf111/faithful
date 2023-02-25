
pub fn decode(data: &mut [u8]) {
    let mut m: u8 = 0;
    for v in &mut data.iter_mut() {
        let k = m.wrapping_sub(3) & 7;
        let mask: u8 = 1 << k;
        *v = !(*v ^ mask);
        m = m.wrapping_add(1);
    }
}
