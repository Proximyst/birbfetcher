use sha2::Digest as _;

pub fn sha256(block: impl FnOnce(&mut sha2::Sha256) -> ()) -> Vec<u8> {
    let mut sha = sha2::Sha256::new();
    block(&mut sha);
    sha.finalize().iter().map(|&b| b).collect()
}
