//! TOTP（RFC 6238）：HMAC-SHA1、30 秒步、6 位码、±1 窗口、防重放

use hmac::{Hmac, Mac};
use rand::RngCore;
use rand::rngs::OsRng;

/// 时间步长（秒）
pub const STEP_SECS: u64 = 30;
/// 动态码位数
pub const CODE_DIGITS: u32 = 6;
/// otpauth:// URI 的 issuer（产品名常量，见 design.md Open Questions）
pub const ISSUER: &str = "Accounting";

type HmacSha1 = Hmac<sha1::Sha1>;

/// 生成 160 位随机 TOTP 密钥。
pub fn generate_secret() -> Vec<u8> {
    let mut secret = vec![0u8; 20];
    OsRng.fill_bytes(&mut secret);
    secret
}

/// 计算指定时间步的 TOTP 码（RFC 6238 动态截断，6 位）。
pub fn totp_at_step(secret: &[u8], step: u64) -> u32 {
    let mut mac = <HmacSha1 as Mac>::new_from_slice(secret).expect("HMAC 接受任意长度密钥");
    mac.update(&step.to_be_bytes());
    let hash = mac.finalize().into_bytes();
    let offset = (hash[19] & 0x0f) as usize;
    let binary = ((u32::from(hash[offset]) & 0x7f) << 24)
        | (u32::from(hash[offset + 1]) << 16)
        | (u32::from(hash[offset + 2]) << 8)
        | u32::from(hash[offset + 3]);
    binary % 10u32.pow(CODE_DIGITS)
}

/// 当前时间步。
pub fn current_step(now_unix: i64) -> u64 {
    now_unix.max(0) as u64 / STEP_SECS
}

/// 验证动态码：允许 ±1 时间步窗口，且拒绝不晚于 `last_step` 的码（防重放）。
/// 验证成功返回命中的时间步，调用方应回写 `last_totp_step`。
pub fn verify_code(
    secret: &[u8],
    code: &str,
    now_unix: i64,
    last_step: Option<i64>,
) -> Option<u64> {
    let code = code.trim();
    if code.len() != CODE_DIGITS as usize || !code.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let Ok(code_num) = code.parse::<u32>() else {
        return None;
    };
    let step = current_step(now_unix);
    // ±1 窗口；step=0 时不下溢
    for candidate in [step.wrapping_sub(1), step, step + 1] {
        if step == 0 && candidate == u64::MAX {
            continue;
        }
        if let Some(last) = last_step
            && candidate <= last as u64
        {
            continue; // 防重放：已用过的时间步不再接受
        }
        if totp_at_step(secret, candidate) == code_num {
            return Some(candidate);
        }
    }
    None
}

/// 生成 otpauth:// URI（供 Authenticator 扫码/导入）。
pub fn otpauth_uri(secret: &[u8], account: &str) -> String {
    format!(
        "otpauth://totp/{issuer}:{account}?secret={secret}&issuer={issuer}&digits={digits}&period={period}",
        issuer = ISSUER,
        account = account,
        secret = base32_encode(secret),
        digits = CODE_DIGITS,
        period = STEP_SECS,
    )
}

/// RFC 4648 base32 编码（无填充，otpauth 惯例）。
pub fn base32_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    let mut out = String::with_capacity(data.len().div_ceil(5) * 8);
    for chunk in data.chunks(5) {
        let mut buf = [0u8; 5];
        buf[..chunk.len()].copy_from_slice(chunk);
        let n = u64::from_be_bytes([0, 0, 0, buf[0], buf[1], buf[2], buf[3], buf[4]]);
        let chars = match chunk.len() {
            1 => 2,
            2 => 4,
            3 => 5,
            4 => 7,
            _ => 8,
        };
        for i in 0..chars {
            let idx = (n >> (35 - 5 * i)) & 0x1f;
            out.push(ALPHABET[idx as usize] as char);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// RFC 6238 附录 B 的 SHA-1 测试密钥（ASCII "12345678901234567890"）
    const RFC_SECRET: &[u8] = b"12345678901234567890";

    /// RFC 6238 已知向量（8 位码取模 10^6 得 6 位期望值）
    #[test]
    fn rfc6238_known_vectors() {
        // (unix 秒, 8 位期望码)
        let cases = [
            (59i64, 94287082u32),
            (1111111109, 7081804),
            (1111111111, 14050471),
            (1234567890, 89005924),
            (2000000000, 69279037),
        ];
        for (t, expected8) in cases {
            let step = current_step(t);
            let got = totp_at_step(RFC_SECRET, step);
            assert_eq!(got, expected8 % 1_000_000, "t={t}");
        }
    }

    #[test]
    fn verify_allows_plus_minus_one_window() {
        let now = 59i64; // step = 1
        let step = current_step(now);
        for s in [step - 1, step, step + 1] {
            let code = format!("{:06}", totp_at_step(RFC_SECRET, s));
            let hit = verify_code(RFC_SECRET, &code, now, None);
            assert_eq!(hit, Some(s));
        }
        // 窗口外（±2）拒绝
        let far = format!("{:06}", totp_at_step(RFC_SECRET, step + 2));
        assert_eq!(verify_code(RFC_SECRET, &far, now, None), None);
    }

    #[test]
    fn verify_rejects_replay() {
        let now = 59i64;
        let step = current_step(now);
        let code = format!("{:06}", totp_at_step(RFC_SECRET, step));
        // 同一时间步已用过 → 拒绝
        assert_eq!(verify_code(RFC_SECRET, &code, now, Some(step as i64)), None);
        // 上一时间步的码也已"过期"（last_step 更晚）→ 拒绝
        let prev = format!("{:06}", totp_at_step(RFC_SECRET, step - 1));
        assert_eq!(verify_code(RFC_SECRET, &prev, now, Some(step as i64)), None);
        // 下一时间步仍可用
        let next = format!("{:06}", totp_at_step(RFC_SECRET, step + 1));
        assert_eq!(
            verify_code(RFC_SECRET, &next, now, Some(step as i64)),
            Some(step + 1)
        );
    }

    #[test]
    fn verify_rejects_malformed() {
        assert_eq!(verify_code(RFC_SECRET, "12345", 59, None), None);
        assert_eq!(verify_code(RFC_SECRET, "12345a", 59, None), None);
        assert_eq!(
            verify_code(RFC_SECRET, "000000", 59, Some(i64::MAX - 1)),
            None
        );
    }

    #[test]
    fn base32_rfc4648_vectors() {
        // RFC 4648 测试向量
        assert_eq!(base32_encode(b""), "");
        assert_eq!(base32_encode(b"f"), "MY");
        assert_eq!(base32_encode(b"fo"), "MZXQ");
        assert_eq!(base32_encode(b"foo"), "MZXW6");
        assert_eq!(base32_encode(b"foob"), "MZXW6YQ");
        assert_eq!(base32_encode(b"fooba"), "MZXW6YTB");
        assert_eq!(base32_encode(b"foobar"), "MZXW6YTBOI");
    }

    #[test]
    fn otpauth_uri_format() {
        let uri = otpauth_uri(b"12345678901234567890", "alice");
        assert!(uri.starts_with("otpauth://totp/Accounting:alice?"));
        assert!(uri.contains("secret=GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ"));
        assert!(uri.contains("issuer=Accounting"));
        assert!(uri.contains("digits=6"));
        assert!(uri.contains("period=30"));
    }

    #[test]
    fn secret_is_160bit_random() {
        let s1 = generate_secret();
        let s2 = generate_secret();
        assert_eq!(s1.len(), 20);
        assert_ne!(s1, s2);
    }
}
