use sparse_merkle_tree::CompiledMerkleProof;
#[inline]
pub fn slice_to_hex_string(slice: &[u8]) -> String {
    let mut s = String::new();
    for i in slice {
        s.push_str(format!("{:02x}", i).as_str());
    }
    //println!("hex = {}", &s);
    return s;
}

#[inline]
pub fn get_empty_compiled_proof() -> CompiledMerkleProof {
    let v: Vec<u8> = Vec::new();
    let compiled_proof = CompiledMerkleProof(v);
    compiled_proof
}
