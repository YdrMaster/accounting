use accounting::error::AccountingError;
use accounting::finance_period::FinancePeriod;
use accounting::id::{AccountId, ChannelId, CommodityId, MemberId};
use accounting_sql::SqliteDatabase;
use rust_decimal::Decimal;
use std::collections::{BTreeMap, HashMap};
use std::str::FromStr;

use super::dto::{
    ConfigFile, Settings, YamlAccount, YamlAccountMapping, YamlAccountOwner, YamlBudget,
    YamlChannel, YamlCommodity, YamlMember, YamlTag,
};

/// 配置导入导出服务
pub struct ConfigService {
    db: SqliteDatabase,
}

impl ConfigService {
    pub fn new(db: SqliteDatabase) -> Self {
        Self { db }
    }

    /// 将数据库中的配置导出为 ConfigFile
    ///
    /// 实体名字按 `lang` 回退链（所选 → en → zh-CN → 插入序）批量解析；
    /// `lang` 同时写入 settings.language，作为回导入时的名字写入语言。
    pub async fn export(&self, lang: &str) -> Result<ConfigFile, AccountingError> {
        let db_err = |e: accounting_sql::DbError| AccountingError::DatabaseError(e.to_string());

        let mut file = ConfigFile {
            settings: Some(Settings {
                language: lang.to_string(),
            }),
            ..Default::default()
        };

        // 实体清单
        let commodities = self.db.commodity_list().await.map_err(db_err)?;
        let members = self.db.member_list().await.map_err(db_err)?;
        let channels = self.db.channel_list().await.map_err(db_err)?;
        let tags = self.db.tag_list().await.map_err(db_err)?;
        let accounts = self.db.account_list().await.map_err(db_err)?;

        // 批量显示名（回退链在包装层完成）
        let commodity_names = self
            .db
            .commodity_display_names(&commodities.iter().map(|c| c.id).collect::<Vec<_>>(), lang)
            .await
            .map_err(db_err)?;
        let member_names = self
            .db
            .member_display_names(&members.iter().map(|m| m.id).collect::<Vec<_>>(), lang)
            .await
            .map_err(db_err)?;
        let channel_names = self
            .db
            .channel_display_names(&channels.iter().map(|c| c.id).collect::<Vec<_>>(), lang)
            .await
            .map_err(db_err)?;
        let tag_names = self
            .db
            .tag_display_names(&tags.iter().map(|t| t.id).collect::<Vec<_>>(), lang)
            .await
            .map_err(db_err)?;
        let account_names = self
            .db
            .account_display_names(&accounts.iter().map(|a| a.id).collect::<Vec<_>>(), lang)
            .await
            .map_err(db_err)?;

        // 账户 ID -> 显示路径
        let accounts_by_id: HashMap<AccountId, accounting::account::Account> =
            accounts.iter().map(|a| (a.id, a.clone())).collect();
        let account_path_map: HashMap<AccountId, String> = accounts
            .iter()
            .map(|a| (a.id, a.display_path(&accounts_by_id, &account_names)))
            .collect();

        // commodities
        file.commodities = commodities
            .iter()
            .map(|c| YamlCommodity {
                symbol: c.symbol.clone(),
                name: commodity_names.get(&c.id).cloned().unwrap_or_default(),
                precision: c.precision,
            })
            .collect();

        // members
        file.members = members
            .iter()
            .map(|m| YamlMember {
                name: member_names.get(&m.id).cloned().unwrap_or_default(),
            })
            .collect();

        // channels
        file.channels = channels
            .iter()
            .map(|c| YamlChannel {
                name: channel_names.get(&c.id).cloned().unwrap_or_default(),
                description: c.description.clone(),
                account: c
                    .account_id
                    .map(|id| account_path_map.get(&id).cloned().unwrap_or_default()),
            })
            .collect();

        // tags
        file.tags = tags
            .iter()
            .map(|t| YamlTag {
                name: tag_names.get(&t.id).cloned().unwrap_or_default(),
                description: t.description.clone(),
            })
            .collect();

        // accounts
        file.accounts = accounts
            .iter()
            .map(|a| YamlAccount {
                path: account_path_map.get(&a.id).cloned().unwrap_or_default(),
                closed_at: a.closed_at.map(|d| d.to_string()),
                billing_day: a.billing_day,
                repayment_day: a.repayment_day,
            })
            .collect();

        // account owners
        let owners = self.db.account_list_owners().await.map_err(db_err)?;
        let mut owners_by_account: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for (account_id, member_id) in owners {
            let account_path = account_path_map
                .get(&account_id)
                .cloned()
                .unwrap_or_default();
            let member_name = member_names.get(&member_id).cloned().unwrap_or_default();
            owners_by_account
                .entry(account_path)
                .or_default()
                .push(member_name);
        }
        file.account_owners = owners_by_account
            .into_iter()
            .map(|(account, members)| YamlAccountOwner { account, members })
            .collect();

        // account mappings
        let mappings = self.db.account_mapping_list_all().await.map_err(db_err)?;
        let mut mappings_by_key: BTreeMap<(String, String), BTreeMap<String, String>> =
            BTreeMap::new();
        for mapping in mappings {
            let member_name = member_names
                .get(&mapping.member_id)
                .cloned()
                .unwrap_or_default();
            let channel_name = channel_names
                .get(&mapping.channel_id)
                .cloned()
                .unwrap_or_default();
            let account_path = account_path_map
                .get(&mapping.account_id)
                .cloned()
                .unwrap_or_default();
            mappings_by_key
                .entry((member_name, channel_name))
                .or_default()
                .insert(mapping.category, account_path);
        }
        file.account_mappings = mappings_by_key
            .into_iter()
            .map(|((member, channel), mappings)| YamlAccountMapping {
                member,
                channel,
                mappings,
            })
            .collect();

        // budgets
        let budgets = self
            .db
            .budget_list_all_with_limits()
            .await
            .map_err(db_err)?;
        let budget_names = self
            .db
            .budget_display_names(&budgets.iter().map(|(b, _)| b.id).collect::<Vec<_>>(), lang)
            .await
            .map_err(db_err)?;
        let commodity_symbol_map: HashMap<CommodityId, String> = commodities
            .iter()
            .map(|c| (c.id, c.symbol.clone()))
            .collect();

        file.budgets = budgets
            .into_iter()
            .map(|(budget, limits)| {
                let commodity = commodity_symbol_map
                    .get(&budget.commodity_id)
                    .cloned()
                    .unwrap_or_default();
                let mut limit_map: BTreeMap<String, String> = BTreeMap::new();
                for limit in limits {
                    let path = account_path_map
                        .get(&limit.account_id)
                        .cloned()
                        .unwrap_or_default();
                    limit_map.insert(path, limit.amount.to_string());
                }
                YamlBudget {
                    name: budget_names.get(&budget.id).cloned().unwrap_or_default(),
                    period: budget.period.to_string(),
                    commodity,
                    limits: limit_map,
                }
            })
            .collect();

        Ok(file)
    }

    /// 将 ConfigFile 导入数据库，按自然键合并更新
    pub async fn import(&self, data: &ConfigFile) -> Result<(), AccountingError> {
        let mut tx = self
            .db
            .transaction()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        // 文件声明的语言作为本次导入所有名字的写入语言
        // （语言不再是数据库属性，不做库内语言一致性检查）
        let file_lang = data
            .settings
            .as_ref()
            .map(|s| s.language.clone())
            .ok_or_else(|| {
                AccountingError::InvalidTransaction("导入文件缺少 settings.language".to_string())
            })?;

        // 缓存：自然键 -> ID
        let mut commodity_id_cache: HashMap<String, CommodityId> = HashMap::new();
        let mut member_id_cache: HashMap<String, MemberId> = HashMap::new();
        let mut channel_id_cache: HashMap<String, ChannelId> = HashMap::new();
        let mut tag_id_cache: HashMap<String, accounting::id::TagId> = HashMap::new();
        let mut account_id_cache: HashMap<String, AccountId> = HashMap::new();

        // 1. commodities
        for c in &data.commodities {
            let id = tx
                .commodity_upsert_by_symbol(&c.symbol, &c.name, c.precision, &file_lang)
                .await
                .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
            commodity_id_cache.insert(c.symbol.clone(), id);
        }

        // 2. members
        for m in &data.members {
            let id = tx
                .member_get_or_create_by_name(&m.name, &file_lang)
                .await
                .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
            member_id_cache.insert(m.name.clone(), id);
        }

        // 3. tags
        for t in &data.tags {
            let id = tx
                .tag_upsert_by_name(&t.name, t.description.as_deref(), &file_lang)
                .await
                .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
            tag_id_cache.insert(t.name.clone(), id);
        }

        // 4. accounts
        for a in &data.accounts {
            let path = &a.path;
            let id = tx
                .account_get_or_create_by_path(path, &file_lang)
                .await
                .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
            account_id_cache.insert(path.clone(), id);

            let closed_at = a
                .closed_at
                .as_ref()
                .map(|s| {
                    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                        .map_err(|e| AccountingError::InvalidDate(e.to_string()))
                })
                .transpose()?;

            tx.account_update_by_path(path, closed_at, a.billing_day, a.repayment_day)
                .await
                .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        }

        // 5. channels
        for c in &data.channels {
            let account_id = c
                .account
                .as_ref()
                .and_then(|path| account_id_cache.get(path).copied());
            let id = tx
                .channel_upsert_by_name(&c.name, c.description.as_deref(), account_id, &file_lang)
                .await
                .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
            channel_id_cache.insert(c.name.clone(), id);
        }

        // 6. account owners
        for owner in &data.account_owners {
            let account_id = *account_id_cache
                .get(&owner.account)
                .ok_or_else(|| AccountingError::AccountNotFound(owner.account.clone()))?;
            let mut member_ids = Vec::new();
            for member_name in &owner.members {
                let member_id = *member_id_cache.get(member_name).ok_or_else(|| {
                    AccountingError::InvalidTransaction(format!("成员不存在: {}", member_name))
                })?;
                member_ids.push(member_id);
            }
            tx.account_set_owners(account_id, &member_ids)
                .await
                .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        }

        // 7. account mappings
        for mapping in &data.account_mappings {
            let member_id = *member_id_cache.get(&mapping.member).ok_or_else(|| {
                AccountingError::InvalidTransaction(format!("成员不存在: {}", mapping.member))
            })?;
            let channel_id = *channel_id_cache.get(&mapping.channel).ok_or_else(|| {
                AccountingError::InvalidTransaction(format!("渠道不存在: {}", mapping.channel))
            })?;

            for (category, account_path) in &mapping.mappings {
                let account_id = *account_id_cache
                    .get(account_path)
                    .ok_or_else(|| AccountingError::AccountNotFound(account_path.clone()))?;
                let model = accounting::account_mapping::AccountMapping {
                    member_id,
                    channel_id,
                    category: category.clone(),
                    account_id,
                };
                tx.account_mapping_upsert(&model)
                    .await
                    .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
            }
        }

        // 8. budgets
        for budget in &data.budgets {
            let period = parse_budget_period(&budget.period)?;
            let commodity_id = *commodity_id_cache
                .get(&budget.commodity)
                .ok_or_else(|| AccountingError::CommodityNotFound(budget.commodity.clone()))?;
            let mut limits = Vec::new();
            for (account_path, amount_str) in &budget.limits {
                let account_id = *account_id_cache
                    .get(account_path)
                    .ok_or_else(|| AccountingError::AccountNotFound(account_path.clone()))?;
                let amount = Decimal::from_str(amount_str).map_err(|e| {
                    AccountingError::InvalidTransaction(format!(
                        "金额解析失败 {}: {}",
                        amount_str, e
                    ))
                })?;
                limits.push((account_id, amount));
            }
            tx.budget_upsert_by_name(&budget.name, period, commodity_id, &limits, &file_lang)
                .await
                .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        }

        // 9. rebuild account ancestors
        tx.account_rebuild_ancestors()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}

fn parse_budget_period(s: &str) -> Result<FinancePeriod, AccountingError> {
    match s {
        "Daily" => Ok(FinancePeriod::Daily),
        "WeeklyFromSunday" => Ok(FinancePeriod::WeeklyFromSunday),
        "WeeklyFromMonday" => Ok(FinancePeriod::WeeklyFromMonday),
        "Monthly" => Ok(FinancePeriod::Monthly),
        "Yearly" => Ok(FinancePeriod::Yearly),
        _ => Err(AccountingError::InvalidTransaction(format!(
            "未知的预算周期: {}",
            s
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::dto::{Settings, YamlAccount, YamlBudget, YamlCommodity};
    use accounting::finance_period::FinancePeriod;
    use accounting::id::CommodityId;
    use rust_decimal::Decimal;
    use std::collections::BTreeMap;

    async fn setup_db() -> SqliteDatabase {
        let db = SqliteDatabase::open_in_memory().await.unwrap();
        db.initialize().await.unwrap();
        db
    }

    fn config_with_lang(lang: &str) -> ConfigFile {
        ConfigFile {
            settings: Some(Settings {
                language: lang.to_string(),
            }),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_export_and_import_roundtrip() {
        let db = setup_db().await;

        // Create some config data
        db.commodity_upsert_by_symbol("USD", "美元", 2, "zh-CN")
            .await
            .unwrap();
        let member_id = db
            .member_get_or_create_by_name("Alice", "en")
            .await
            .unwrap();
        let _tag_id = db.tag_upsert_by_name("trip", None, "en").await.unwrap();
        let account_id = db
            .account_get_or_create_by_path("Assets:Bank:Checking", "en")
            .await
            .unwrap();
        db.account_update_by_path("Assets:Bank:Checking", None, Some(5), Some(25))
            .await
            .unwrap();
        let channel_id = db
            .channel_upsert_by_name("微信", Some("测试"), Some(account_id), "zh-CN")
            .await
            .unwrap();
        db.account_set_owners(account_id, &[member_id])
            .await
            .unwrap();

        let mapping = accounting::account_mapping::AccountMapping {
            member_id,
            channel_id,
            category: "收支:餐饮美食".to_string(),
            account_id,
        };
        db.account_mapping_upsert(&mapping).await.unwrap();

        db.budget_upsert_by_name(
            "月度预算",
            FinancePeriod::Monthly,
            CommodityId(1),
            &[(account_id, Decimal::from_str("3000.00").unwrap())],
            "zh-CN",
        )
        .await
        .unwrap();

        // Export
        let service = ConfigService::new(db.clone());
        let exported = service.export("en").await.unwrap();

        assert_eq!(exported.version, "1.0");
        assert!(
            exported
                .commodities
                .iter()
                .any(|c| c.symbol == "USD" && c.name == "美元")
        );
        assert!(exported.members.iter().any(|m| m.name == "Alice"));
        assert!(exported.tags.iter().any(|t| t.name == "trip"));
        assert!(
            exported
                .accounts
                .iter()
                .any(|a| a.path == "Assets:Bank:Checking")
        );
        assert!(exported.channels.iter().any(|c| c.name == "微信"));
        assert_eq!(exported.account_owners.len(), 1);
        assert_eq!(exported.account_mappings.len(), 1);
        assert_eq!(exported.budgets.len(), 1);

        // Import into a fresh database
        let db2 = setup_db().await;
        let service2 = ConfigService::new(db2);
        service2.import(&exported).await.unwrap();

        // Verify imported data
        let imported = service2.export("en").await.unwrap();
        assert!(
            imported
                .commodities
                .iter()
                .any(|c| c.symbol == "USD" && c.name == "美元")
        );
        assert!(imported.members.iter().any(|m| m.name == "Alice"));
        assert!(imported.tags.iter().any(|t| t.name == "trip"));
        assert!(
            imported
                .accounts
                .iter()
                .any(|a| a.path == "Assets:Bank:Checking")
        );
        assert!(imported.channels.iter().any(|c| c.name == "微信"));
        assert_eq!(imported.account_owners.len(), 1);
        assert_eq!(imported.account_mappings.len(), 1);
        assert_eq!(imported.budgets.len(), 1);
    }

    #[tokio::test]
    async fn test_import_auto_creates_parent_accounts() {
        let db = setup_db().await;
        let mut config = config_with_lang("zh-CN");
        config.accounts.push(YamlAccount {
            path: "Assets:Bank:Checking".to_string(),
            closed_at: None,
            billing_day: None,
            repayment_day: None,
        });

        let service = ConfigService::new(db.clone());
        service.import(&config).await.unwrap();

        // 路径各层级均可按名字命中
        assert!(db.account_get_by_name("Assets").await.unwrap().is_some());
        assert!(
            db.account_get_by_name("Assets:Bank")
                .await
                .unwrap()
                .is_some()
        );
        assert!(
            db.account_get_by_name("Assets:Bank:Checking")
                .await
                .unwrap()
                .is_some()
        );
    }

    #[tokio::test]
    async fn test_import_rollback_on_error() {
        let db = setup_db().await;
        let mut config = config_with_lang("zh-CN");
        config.commodities.push(YamlCommodity {
            symbol: "USD".to_string(),
            name: "美元".to_string(),
            precision: 2,
        });
        // Invalid budget: references non-existent commodity
        config.budgets.push(YamlBudget {
            name: "错误预算".to_string(),
            period: "Monthly".to_string(),
            commodity: "XXX".to_string(),
            limits: BTreeMap::new(),
        });

        let service = ConfigService::new(db.clone());
        let result = service.import(&config).await;
        assert!(result.is_err());

        // Verify commodity was not committed
        assert!(db.commodity_get_by_symbol("USD").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_import_missing_language_rejected() {
        let db = setup_db().await;
        let config = ConfigFile::default();

        let service = ConfigService::new(db);
        let result = service.import(&config).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("settings.language")
        );
    }

    #[tokio::test]
    async fn test_import_merge_update() {
        let db = setup_db().await;
        db.commodity_upsert_by_symbol("USD", "美元", 2, "zh-CN")
            .await
            .unwrap();

        let mut config = config_with_lang("zh-CN");
        config.commodities.push(YamlCommodity {
            symbol: "USD".to_string(),
            name: "US Dollar".to_string(),
            precision: 4,
        });

        let service = ConfigService::new(db.clone());
        service.import(&config).await.unwrap();

        // 合并语义：同 symbol 更新精度（名字不做覆盖更新）
        let usd = db.commodity_get_by_symbol("USD").await.unwrap().unwrap();
        assert_eq!(usd.precision, 4);
    }

    #[tokio::test]
    async fn test_account_path_change_creates_new_account() {
        let db = setup_db().await;
        db.account_get_or_create_by_path("Assets:Bank:Checking", "zh-CN")
            .await
            .unwrap();

        let mut config = config_with_lang("zh-CN");
        config.accounts.push(YamlAccount {
            path: "Assets:Bank:SalaryCard".to_string(),
            closed_at: None,
            billing_day: None,
            repayment_day: None,
        });

        let service = ConfigService::new(db.clone());
        service.import(&config).await.unwrap();

        // 旧路径保留，新路径创建
        assert!(
            db.account_get_by_name("Assets:Bank:Checking")
                .await
                .unwrap()
                .is_some()
        );
        assert!(
            db.account_get_by_name("Assets:Bank:SalaryCard")
                .await
                .unwrap()
                .is_some()
        );
    }
}
