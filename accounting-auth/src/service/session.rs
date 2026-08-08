//! 不透明 session token：256 位随机 + SHA-256 哈希 + 滑动过期

use rand::RngCore;
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};

/// 正式 session 有效期：7 天（滑动过期）
pub const SESSION_TTL_SECS: i64 = 7 * 24 * 3600;
/// pending session（等待 TOTP 第二步）有效期：5 分钟
pub const PENDING_TTL_SECS: i64 = 5 * 60;

/// 生成 256 位随机 session token（hex 编码，64 字符）。
pub fn generate_token() -> String {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    to_hex(&bytes)
}

/// 计算 token 的 SHA-256 哈希（hex）。DB 只存哈希不存原文。
pub fn hash_token(token: &str) -> String {
    to_hex(&Sha256::digest(token.as_bytes()))
}

/// 判断会话是否已过期。
pub fn is_expired(expires_at: i64, now: i64) -> bool {
    now >= expires_at
}

/// 字节串转小写 hex。
pub(crate) fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_is_256bit_random_hex() {
        let t1 = generate_token();
        let t2 = generate_token();
        assert_eq!(t1.len(), 64);
        assert!(t1.chars().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(t1, t2, "两次生成的 token 不应相同");
    }

    #[test]
    fn hash_is_sha256_hex() {
        // SHA-256("abc") 已知值
        let h = hash_token("abc");
        assert_eq!(
            h,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn expiry_check() {
        assert!(!is_expired(1000, 999));
        assert!(is_expired(1000, 1000));
        assert!(is_expired(1000, 1001));
    }
}
