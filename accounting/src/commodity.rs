use crate::id::CommodityId;

/// 商品/货币
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Commodity {
    /// 商品唯一标识符
    pub id: CommodityId,
    /// 符号，如 CNY、USD
    pub symbol: String,
    /// 小数精度
    pub precision: u8,
    /// 创建日期
    pub created_at: Option<chrono::NaiveDate>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::CommodityId;

    #[test]
    fn test_commodity_fields() {
        let c = Commodity {
            id: CommodityId(1),
            symbol: "CNY".to_string(),
            precision: 2,
            created_at: None,
        };
        assert_eq!(c.symbol, "CNY");
        assert_eq!(c.precision, 2);
    }
}
