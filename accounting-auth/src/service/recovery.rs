//! 一次性恢复码：8 个，存 SHA-256 哈希，用过即作废

use rand::RngCore;
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};

use super::session::to_hex;
use crate::error::{AuthError, Result};

/// 恢复码数量
pub const RECOVERY_CODE_COUNT: usize = 8;

/// 生成 8 个恢复码明文（形如 `ab12-cd34-ef56`，仅 enable 时返回一次）。
pub fn generate_codes() -> Vec<String> {
    let mut bytes = [0u8; 6];
    (0..RECOVERY_CODE_COUNT)
        .map(|_| {
            OsRng.fill_bytes(&mut bytes);
            let h = to_hex(&bytes);
            format!("{}-{}-{}", &h[0..4], &h[4..8], &h[8..12])
        })
        .collect()
}

/// 恢复码的存储形式：SHA-256 哈希。
pub fn hash_code(code: &str) -> String {
    to_hex(&Sha256::digest(normalize(code).as_bytes()))
}

/// 归一化：去空格/连字符、转小写，容忍用户输入差异。
fn normalize(code: &str) -> String {
    code.chars()
        .filter(|c| !c.is_whitespace() && *c != '-')
        .flat_map(|c| c.to_lowercase())
        .collect()
}

/// 哈希列表的 JSON 序列化（存 users.recovery_codes）。
pub fn hashes_to_json(hashes: &[String]) -> String {
    serde_json::to_string(hashes).expect("Vec<String> 序列化不会失败")
}

/// 从 JSON 解析哈希列表。
pub fn hashes_from_json(json: &str) -> Result<Vec<String>> {
    serde_json::from_str(json).map_err(|e| AuthError::Config(format!("恢复码 JSON 损坏: {e}")))
}

/// 消费一个恢复码：在哈希列表中查找并移除，返回更新后的 JSON。
/// 码无效或不在列表中时返回 None。
pub fn consume_code(codes_json: &str, code: &str) -> Option<String> {
    let mut hashes = hashes_from_json(codes_json).ok()?;
    let target = hash_code(code);
    let pos = hashes.iter().position(|h| *h == target)?;
    hashes.remove(pos);
    Some(hashes_to_json(&hashes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_8_unique_formatted_codes() {
        let codes = generate_codes();
        assert_eq!(codes.len(), 8);
        for c in &codes {
            assert_eq!(c.len(), 14); // xxxx-xxxx-xxxx
            assert_eq!(c.chars().filter(|c| *c == '-').count(), 2);
        }
        let mut sorted = codes.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), 8, "恢复码不应重复");
    }

    #[test]
    fn consume_is_one_time() {
        let codes = generate_codes();
        let json = hashes_to_json(&codes.iter().map(|c| hash_code(c)).collect::<Vec<_>>());

        // 消费一个码
        let json2 = consume_code(&json, &codes[0]).expect("有效恢复码应通过");
        assert_eq!(hashes_from_json(&json2).unwrap().len(), 7);

        // 同一码再次提交 → 失败（一次性）
        assert!(consume_code(&json2, &codes[0]).is_none());
        // 无效码 → 失败
        assert!(consume_code(&json2, "0000-0000-0000").is_none());
        // 其余码仍可用；输入容忍大小写与分隔符差异
        let upper = codes[1].to_uppercase().replace('-', "");
        assert!(consume_code(&json2, &upper).is_some());
    }
}
