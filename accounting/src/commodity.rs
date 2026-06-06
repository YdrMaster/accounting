use crate::id::CommodityId;

/// 商品/货币
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Commodity {
    /// 商品唯一标识符
    pub id: CommodityId,
    /// 符号，如 CNY、USD
    pub symbol: String,
    /// 商品名称
    pub name: String,
    /// 小数精度
    pub precision: u8,
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
            name: "人民币".to_string(),
            precision: 2,
        };
        assert_eq!(c.symbol, "CNY");
        assert_eq!(c.precision, 2);
    }
}
