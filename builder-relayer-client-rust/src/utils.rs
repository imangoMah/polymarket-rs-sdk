use ethers::types::{Address, U256};
use sha3::{Digest, Keccak256};

pub fn sleep_ms(ms: u64) -> tokio::time::Sleep {
    tokio::time::sleep(std::time::Duration::from_millis(ms))
}

pub fn keccak256(bytes: &[u8]) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    hasher.update(bytes);
    let res = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&res);
    out
}

pub fn hex_to_address(s: &str) -> Address {
    s.parse().expect("invalid address")
}

pub fn u256_from_dec_str(s: &str) -> U256 {
    U256::from_dec_str(s).expect("invalid u256")
}

/// Split and repack ECDSA signature like TS splitAndPackSig
/// input: 0x{r}{s}{v}
/// output: 0x{r(32)}{s(32)}{v(1)}
pub fn split_and_pack_sig(sig_hex: &str) -> String {
    // Mirror TS logic (splitSignature + encodePacked) so relayer validates:
    // TS maps: 0/1 -> 31/32 and 27/28 -> 31/32 then encodes r,s,v as uint256,uint256,uint8 (decimal bigints)
    let s = sig_hex.trim_start_matches("0x");
    assert!(s.len() >= 130, "sig too short");
    let r_hex = &s[0..64];
    let s_hex = &s[64..128];
    let v_hex = &s[128..130];
    let mut v = u8::from_str_radix(v_hex, 16).expect("v");
    match v {
        0 | 1 => v += 31,  // 0/1 -> 31/32
        27 | 28 => v += 4, // 27/28 -> 31/32
        _ => {}
    }
    // r,s interpreted as uint256 decimal strings in TS. We'll keep hex then convert to U256 -> decimal.
    use ethers::types::U256;
    let r_u256 = U256::from_str_radix(r_hex, 16).expect("r parse");
    let s_u256 = U256::from_str_radix(s_hex, 16).expect("s parse");
    // encodePacked(uint256,uint256,uint8) == left padded 32 bytes for r,s then 1 byte v
    // Simpler: manually pad r,s as 32-byte big endian.
    let mut r_bytes = [0u8; 32];
    let mut s_bytes = [0u8; 32];
    r_u256.to_big_endian(&mut r_bytes);
    s_u256.to_big_endian(&mut s_bytes);
    let packed_bytes: Vec<u8> = [r_bytes.as_slice(), s_bytes.as_slice(), &[v]].concat();
    format!("0x{}", hex::encode(packed_bytes))
}
