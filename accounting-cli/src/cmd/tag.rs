use crate::cmd::TagRow;
use crate::output::{OutputFormat, print_line, print_vec};
use accounting_sql::SqliteDatabase;
use clap::{Args, Subcommand};
use rust_i18n::t;

#[derive(Subcommand)]
pub enum TagCmd {
    /// 列出标签
    List,
    /// 添加标签
    Add(TagAddArgs),
    /// 删除标签
    Delete(TagDeleteArgs),
    /// 改名（按 --lang 指定的语言设置/新增显示名）
    Rename(TagRenameArgs),
}

#[derive(Args)]
pub struct TagAddArgs {
    pub name: String,
    #[arg(long)]
    pub description: Option<String>,
}

#[derive(Args)]
pub struct TagDeleteArgs {
    pub name: String,
}

#[derive(Args)]
pub struct TagRenameArgs {
    /// 标签名（现有任意名字均可命中）
    pub name: String,
    /// 新名字（按 --lang 语言写入；该语言无显示名时新增显示名）
    pub new_name: String,
}

impl TagCmd {
    pub async fn run(
        self,
        db: SqliteDatabase,
        format: OutputFormat,
        lang: &str,
    ) -> Result<(), accounting::error::AccountingError> {
        match self {
            TagCmd::List => {
                let service = accounting_service::tag_service::TagService::new(db.clone());
                let tags = service.list().await?;
                let ids: Vec<accounting::id::TagId> = tags.iter().map(|t| t.id).collect();
                let names = db.tag_display_names(&ids, lang).await.map_err(|e| {
                    accounting::error::AccountingError::DatabaseError(e.to_string())
                })?;
                // 系统标签的英文系统名，用作描述翻译键
                let system_ids: Vec<accounting::id::TagId> =
                    tags.iter().filter(|t| t.is_system).map(|t| t.id).collect();
                let en_names = db.tag_display_names(&system_ids, "en").await.map_err(|e| {
                    accounting::error::AccountingError::DatabaseError(e.to_string())
                })?;
                let rows: Vec<TagRow> = tags
                    .iter()
                    .map(|t| {
                        let mut row = TagRow::new(t, names.get(&t.id).cloned().unwrap_or_default());
                        if t.is_system {
                            if let Some(desc) = en_names.get(&t.id).and_then(|n| {
                                accounting_service::tag_service::system_tag_description(n, lang)
                            }) {
                                row.description = desc;
                            }
                        }
                        row
                    })
                    .collect();
                print_vec(&rows, format);
            }
            TagCmd::Add(args) => {
                let service = accounting_service::tag_service::TagService::new(db);
                let id = service
                    .add(args.name.clone(), args.description, lang)
                    .await?;
                print_line(&format!("{}", t!("tag_created", id = id.0)), format);
            }
            TagCmd::Delete(args) => {
                let service = accounting_service::tag_service::TagService::new(db);
                service.delete(&args.name).await?;
                print_line(&format!("{}", t!("tag_deleted", name = args.name)), format);
            }
            TagCmd::Rename(args) => {
                let tag = db
                    .tag_get_by_name(&args.name)
                    .await
                    .map_err(|e| accounting::error::AccountingError::DatabaseError(e.to_string()))?
                    .ok_or_else(|| {
                        accounting::error::AccountingError::InvalidTransaction(format!(
                            "{}",
                            t!("tag_name_not_found", name = args.name)
                        ))
                    })?;
                db.tag_update(tag.id, &args.new_name, tag.description.as_deref(), lang)
                    .await
                    .map_err(|e| {
                        accounting::error::AccountingError::DatabaseError(e.to_string())
                    })?;
                print_line(
                    &format!(
                        "{}",
                        t!("tag_renamed", name = args.name, new_name = args.new_name)
                    ),
                    format,
                );
            }
        }
        Ok(())
    }
}
