//! argon2id 密码哈希与校验

use crate::error::{AuthError, Result};
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use rand::rngs::OsRng;

/// 计算密码的 argon2id 哈希（默认参数，含随机盐）。
pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| AuthError::PasswordHash(e.to_string()))
}

/// 校验密码是否匹配哈希。
pub fn verify_password(hash: &str, password: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok()
}

/// 用户不存在时用于垫时间的一致化哈希校验，
/// 使"用户不存在"与"密码错误"耗时接近，防用户名枚举。
/// 该哈希对应一个不可能被猜中的随机密码，verify 必然失败。
pub fn dummy_verify(password: &str) {
    use std::sync::OnceLock;
    // 进程内惰性生成一次的 argon2id 哈希（对应一个不可能被猜中的口令）
    static DUMMY_HASH: OnceLock<String> = OnceLock::new();
    let hash = DUMMY_HASH
        .get_or_init(|| hash_password("dummy-password-for-constant-time").expect("dummy hash"));
    let _ = verify_password(hash, password);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_and_verify_roundtrip() {
        let hash = hash_password("s3cret密码").unwrap();
        assert!(hash.starts_with("$argon2id$"));
        assert!(verify_password(&hash, "s3cret密码"));
        assert!(!verify_password(&hash, "wrong"));
    }

    #[test]
    fn same_password_different_salt() {
        let h1 = hash_password("abc").unwrap();
        let h2 = hash_password("abc").unwrap();
        assert_ne!(h1, h2);
    }

    #[test]
    fn dummy_verify_always_fails() {
        dummy_verify("whatever"); // 不 panic
        assert!(!verify_password("not-a-hash", "x"));
    }
}
