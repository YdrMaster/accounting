use crate::id::CommodityId;

/// 商品/货币
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Commodity {
    pub id: CommodityId,
    pub symbol: String,
    pub name: String,
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
