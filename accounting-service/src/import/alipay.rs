use super::{AdaptError, BillAdapter, BillEntry, BillPosting};
use accounting::transaction::TransactionKind;
use chrono::NaiveDateTime;
use encoding_rs::GBK;
use rust_decimal::Decimal;
use std::str::FromStr;

/// 支付宝账单适配器
pub struct AlipayAdapter;

impl BillAdapter for AlipayAdapter {
    fn names(&self) -> &[&str] {
        &["alipay", "支付宝"]
    }

    fn parse<'a>(
        &'a self,
        data: &[u8],
        ctx: &super::ImportContext,
    ) -> Result<Box<dyn Iterator<Item = Result<BillEntry, AdaptError>> + 'a>, AdaptError> {
        // 支付宝导出为 GBK 编码，先尝试 GBK 解码，失败回退 UTF-8
        let (text, _, had_errors) = GBK.decode(data);
        let text = if had_errors {
            // GBK 解码失败，尝试 UTF-8（测试数据或新版导出格式）
            std::str::from_utf8(data)
                .map_err(|e| {
                    AdaptError::FormatError(format!("文件编码无法识别（GBK/UTF-8 均失败）：{e}"))
                })?
                .to_string()
        } else {
            text.into_owned()
        };

        Ok(Box::new(AlipayIterator {
            lines: text.lines().map(|l| l.to_string()).collect(),
            pos: 0,
            import_root: ctx.import_root.clone(),
        }))
    }
}

struct AlipayIterator {
    lines: Vec<String>,
    pos: usize,
    import_root: String,
}

impl Iterator for AlipayIterator {
    type Item = Result<BillEntry, AdaptError>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.pos < self.lines.len() {
            let line = &self.lines[self.pos];
            let trimmed = line.trim();

            // 跳过空行
            if trimmed.is_empty() {
                self.pos += 1;
                continue;
            }

            // 支付宝 CSV 表头特征：以 "交易时间,交易分类" 开头
            if trimmed.starts_with("交易时间,交易分类") {
                self.pos += 1;
                continue;
            }

            // 尝试按逗号分割，数据行至少需要 12 列
            let fields = split_csv_line(trimmed);
            if fields.len() < 12 {
                self.pos += 1;
                continue;
            }

            // 检查第一个字段是否看起来像日期（数据行特征）
            let date_field = fields[0].trim();
            if !date_field.contains('-') && !date_field.contains('/') {
                self.pos += 1;
                continue;
            }

            let row_num = self.pos + 1;
            self.pos += 1;

            return Some(parse_alipay_row(row_num, &fields, &self.import_root));
        }

        None
    }
}

/// 解析单行支付宝账单
///
/// 支付宝 CSV 导出格式（2026 年版本）：
/// 交易时间, 交易分类, 交易对方, 对方账号, 商品说明, 收/支, 金额, 收/付款方式, 交易状态, 交易订单号, 商家订单号, 备注
/// 索引:  0          1         2         3         4       5     6      7          8         9            10        11
fn parse_alipay_row(
    row: usize,
    fields: &[&str],
    import_root: &str,
) -> Result<BillEntry, AdaptError> {
    let field = |idx: usize, name: &str| -> Result<&str, AdaptError> {
        fields
            .get(idx)
            .ok_or_else(|| AdaptError::RowError {
                row,
                message: format!("缺少 {name} 列（索引 {idx}）"),
            })
            .map(|s| s.trim())
    };

    // 交易时间（索引 0）
    let date_str = field(0, "交易时间")?;
    let date_time = parse_datetime(row, date_str)?;

    // 交易分类（索引 1）— 直接用作 Import 子账户名
    let category = field(1, "交易分类")?;

    // 交易对方（索引 2）
    let counterparty = field(2, "交易对方")?;

    // 商品说明（索引 4）
    let product = field(4, "商品说明")?;

    // 收/支（索引 5）
    let direction = field(5, "收/支")?;

    // 金额（索引 6）
    let amount_str = field(6, "金额")?;
    let amount = Decimal::from_str(amount_str).map_err(|e| AdaptError::RowError {
        row,
        message: format!("金额解析失败 '{amount_str}'：{e}"),
    })?;

    // 交易状态（索引 8）
    let status = field(8, "交易状态")?;

    // 跳过已关闭的交易
    if status.contains("交易关闭") || status.contains("关闭") {
        return Err(AdaptError::RowError {
            row,
            message: "交易已关闭".to_string(),
        });
    }

    // 判断交易类型
    let kind = if category == "退款" {
        TransactionKind::Refund
    } else {
        TransactionKind::Normal
    };

    // 判断金额方向
    let signed_amount = if kind == TransactionKind::Refund {
        // 退款始终为正值（资金收回）
        amount.abs()
    } else if direction.contains("支") {
        -amount.abs()
    } else if direction.contains("收") {
        amount.abs()
    } else {
        // 不计收支（投资理财、转账、信用借贷等）
        // 金额为正表示资金流出（买理财/还款），为负表示流入（赎回/借款）
        // 统一取绝对值，方向由用户后续确认
        -amount.abs()
    };

    // 描述：组合对方和商品名
    let description = if counterparty.is_empty() {
        product.to_string()
    } else if product.is_empty() || product == "/" {
        counterparty.to_string()
    } else {
        format!("{counterparty} - {product}")
    };

    // 构建 BillPosting
    let expense_path = format!("{import_root}:支付宝:{category}");
    let asset_path = format!("{import_root}:支付宝");
    let commodity_symbol = "CNY".to_string();

    let postings = vec![
        BillPosting {
            account_path: expense_path,
            commodity_symbol: commodity_symbol.clone(),
            amount: signed_amount,
            is_reimbursable: false,
        },
        BillPosting {
            account_path: asset_path,
            commodity_symbol,
            amount: -signed_amount,
            is_reimbursable: false,
        },
    ];

    Ok(BillEntry {
        date_time,
        description,
        kind,
        postings,
        tags: vec![],
        row: Some(row),
    })
}

/// 解析日期时间字符串
fn parse_datetime(row: usize, s: &str) -> Result<NaiveDateTime, AdaptError> {
    if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return Ok(dt);
    }
    if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y/%m/%d %H:%M:%S") {
        return Ok(dt);
    }
    Err(AdaptError::RowError {
        row,
        message: format!("日期格式无法解析：'{s}'"),
    })
}

/// CSV 行分割（按逗号，处理末尾 tab 字符）
fn split_csv_line(line: &str) -> Vec<&str> {
    line.split(',').map(|s| s.trim_end_matches('\t')).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use accounting::id::{ChannelId, CommodityId, MemberId};
    use std::str::FromStr;

    fn test_context() -> crate::import::ImportContext {
        crate::import::ImportContext {
            member_id: MemberId(1),
            channel_id: ChannelId(1),
            commodity_id: CommodityId(1),
            import_root: "Import".to_string(),
        }
    }

    #[test]
    fn test_alipay_adapter_names() {
        let adapter = AlipayAdapter;
        assert_eq!(adapter.names(), &["alipay", "支付宝"]);
    }

    #[test]
    fn test_parse_real_format_csv() {
        let adapter = AlipayAdapter;
        let ctx = test_context();

        // 模拟真实支付宝导出格式（GBK 编码的 UTF-8 等价）
        let csv_data = concat!(
            "交易时间,交易分类,交易对方,对方账号,商品说明,收/支,金额,收/付款方式,交易状态,交易订单号,商家订单号,备注,\n",
            "2026-06-25 12:24:45,餐饮美食,茶百道,sxz***@126.com,【茶百道】黄金百香芒芒,支出,4.80,蚂蚁宝藏信用卡,交易成功,2026062522001470791431356972\t,20260625016000001307976921929390\t,,\n",
            "2026-06-25 11:07:07,保险,中国大地财产保险股份有限公司,/,2026.6月保费缴清,支出,18.00,蚂蚁宝藏信用卡,交易成功,20260625102553010470790012776788\t,202606251100300706380154262378\t,,\n",
        );

        let result = adapter.parse(csv_data.as_bytes(), &ctx);
        assert!(result.is_ok());

        let mut iter = result.unwrap();
        let entry1 = iter.next().unwrap().unwrap();
        assert_eq!(entry1.description, "茶百道 - 【茶百道】黄金百香芒芒");
        assert_eq!(entry1.kind, TransactionKind::Normal);
        assert_eq!(entry1.postings.len(), 2);
        assert_eq!(
            entry1.postings[0].amount,
            Decimal::from_str("-4.80").unwrap()
        );
        assert_eq!(entry1.postings[0].account_path, "Import:支付宝:餐饮美食");
        assert_eq!(
            entry1.postings[1].amount,
            Decimal::from_str("4.80").unwrap()
        );
        assert_eq!(entry1.postings[1].account_path, "Import:支付宝");

        let entry2 = iter.next().unwrap().unwrap();
        assert_eq!(
            entry2.description,
            "中国大地财产保险股份有限公司 - 2026.6月保费缴清"
        );
        assert_eq!(entry2.postings[0].account_path, "Import:支付宝:保险");
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_parse_refund_row() {
        let adapter = AlipayAdapter;
        let ctx = test_context();

        let csv_data = concat!(
            "交易时间,交易分类,交易对方,对方账号,商品说明,收/支,金额,收/付款方式,交易状态,交易订单号,商家订单号,备注,\n",
            "2026-06-21 22:56:53,退款,恒源**店,365***@qq.com,退款-透明防摔手机壳,不计收支,36.75,招商银行信用卡,退款成功,2026061723001170791442209186_xxx\t,T200P3308367722981006462\t,,\n",
        );

        let mut iter = adapter.parse(csv_data.as_bytes(), &ctx).unwrap();
        let entry = iter.next().unwrap().unwrap();
        assert_eq!(entry.kind, TransactionKind::Refund);
        // 退款金额为正值，支出侧为正（退款收回），资产侧为负
        assert_eq!(
            entry.postings[0].amount,
            Decimal::from_str("36.75").unwrap()
        );
        assert_eq!(entry.postings[0].account_path, "Import:支付宝:退款");
    }

    #[test]
    fn test_parse_closed_transaction_skipped() {
        let adapter = AlipayAdapter;
        let ctx = test_context();

        let csv_data = concat!(
            "交易时间,交易分类,交易对方,对方账号,商品说明,收/支,金额,收/付款方式,交易状态,交易订单号,商家订单号,备注,\n",
            "2026-06-18 20:17:07,日用百货,恒源**店,jia***@dongfanghongma.com,马桶疏通器,支出,15.90,花呗,交易关闭,2026061823001170791449142701\t,T200P3308618139232033290\t,,\n",
        );

        let mut iter = adapter.parse(csv_data.as_bytes(), &ctx).unwrap();
        let result = iter.next().unwrap();
        assert!(result.is_err(), "交易关闭应返回错误（被跳过）");
    }

    #[test]
    fn test_parse_skips_metadata_header() {
        let adapter = AlipayAdapter;
        let ctx = test_context();

        // 模拟真实支付宝导出文件的完整头部
        let csv_data = concat!(
            "------------------------------------------------------------------------------------\n",
            "导出信息：\n",
            "姓名：杨德睿\n",
            "支付宝账户：ydrml@163.com\n",
            "起始时间：[2026-05-25 00:00:00]    终止时间：[2026-06-25 23:59:59]\n",
            "导出交易类型：[全部]\n",
            "导出时间：[2026-06-25 16:54:22]\n",
            "共354笔记录\n",
            "收入：6笔 23552.22元\n",
            "支出：224笔 15587.62元\n",
            "不计收支：124笔 265854.12元\n",
            "\n",
            "特别提示：\n",
            "1.本回单内容可表明支付宝受理了相应支付交易申请...\n",
            "\n",
            "------------------------支付宝支付科技有限公司  电子客户回单------------------------\n",
            "交易时间,交易分类,交易对方,对方账号,商品说明,收/支,金额,收/付款方式,交易状态,交易订单号,商家订单号,备注,\n",
            "2026-06-25 12:24:45,餐饮美食,茶百道,sxz***@126.com,【茶百道】黄金百香芒芒,支出,4.80,蚂蚁宝藏信用卡,交易成功,2026062522001470791431356972\t,20260625016000001307976921929390\t,,\n",
        );

        let mut iter = adapter.parse(csv_data.as_bytes(), &ctx).unwrap();
        let entry = iter.next().unwrap().unwrap();
        assert_eq!(entry.description, "茶百道 - 【茶百道】黄金百香芒芒");
        assert_eq!(entry.postings[0].account_path, "Import:支付宝:餐饮美食");
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_parse_not_income_expense() {
        let adapter = AlipayAdapter;
        let ctx = test_context();

        // 不计收支类型（投资理财、转账等）
        let csv_data = concat!(
            "交易时间,交易分类,交易对方,对方账号,商品说明,收/支,金额,收/付款方式,交易状态,交易订单号,商家订单号,备注,\n",
            "2026-06-25 10:48:04,投资理财,蚂蚁财富,/,蚂蚁财富-鹏华丰禄债券-买入,不计收支,100.00,余额宝,付款成功,20260625901080012204790009159615\t,\t,,\n",
        );

        let mut iter = adapter.parse(csv_data.as_bytes(), &ctx).unwrap();
        let entry = iter.next().unwrap().unwrap();
        assert_eq!(entry.kind, TransactionKind::Normal);
        assert_eq!(entry.postings[0].account_path, "Import:支付宝:投资理财");
        assert_eq!(
            entry.postings[0].amount,
            Decimal::from_str("-100.00").unwrap()
        );
    }

    #[test]
    fn test_parse_real_file() {
        let adapter = AlipayAdapter;
        let ctx = test_context();
        let data = std::fs::read("test/支付宝交易明细.csv").unwrap();
        let iter = adapter.parse(&data, &ctx).unwrap();
        let mut count = 0;
        let mut errors = 0;
        let mut categories = std::collections::HashSet::new();
        for entry in iter {
            match entry {
                Ok(e) => {
                    count += 1;
                    categories.insert(e.postings[0].account_path.clone());
                }
                Err(e) => {
                    errors += 1;
                    if errors <= 3 {
                        eprintln!("skip: {}", e);
                    }
                }
            }
        }
        eprintln!("成功: {count} 条, 跳过: {errors} 条");
        eprintln!("分类: {:?}", categories);
        assert!(
            count > 0,
            "应至少导入一条记录，实际成功 {count}，跳过 {errors}"
        );
        // 354 条记录中，交易关闭的会被跳过，预期成功 > 300
        assert!(count > 300, "预期成功 300+ 条，实际 {count}");
    }
}
