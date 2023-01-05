#[inline]
pub fn slice_to_hex_string(slice: &[u8]) -> String {
    let mut s = String::new();
    for i in slice {
        s.push_str(format!("{:02x}", i).as_str());
    }
    println!("hex = {}", &s);
    return s;
}
