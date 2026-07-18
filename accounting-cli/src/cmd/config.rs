use accounting_service::config::{ConfigFile, ConfigService};
use accounting_sql::SqliteDatabase;
use clap::{Args, Subcommand};
use rust_i18n::t;
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum ConfigCmd {
    /// 导出配置到 YAML 文件
    Export(ConfigExportArgs),
    /// 从 YAML 文件导入配置
    Import(ConfigImportArgs),
}

#[derive(Args)]
pub struct ConfigExportArgs {
    /// 输出文件路径
    pub file: PathBuf,
}

#[derive(Args)]
pub struct ConfigImportArgs {
    /// 输入文件路径
    pub file: PathBuf,
}

impl ConfigCmd {
    pub async fn run(
        self,
        db: SqliteDatabase,
        lang: &str,
    ) -> Result<(), accounting::error::AccountingError> {
        match self {
            ConfigCmd::Export(args) => {
                let service = ConfigService::new(db);
                let config = service.export(lang).await?;
                let yaml = serde_yaml::to_string(&config)
                    .map_err(|e| accounting::error::AccountingError::Unknown(e.to_string()))?;
                std::fs::write(&args.file, yaml)
                    .map_err(|e| accounting::error::AccountingError::Unknown(e.to_string()))?;
                println!("{}", t!("config_exported", file = args.file.display()));
            }
            ConfigCmd::Import(args) => {
                let yaml = std::fs::read_to_string(&args.file)
                    .map_err(|e| accounting::error::AccountingError::Unknown(e.to_string()))?;
                let config: ConfigFile = serde_yaml::from_str(&yaml).map_err(|e| {
                    accounting::error::AccountingError::InvalidTransaction(e.to_string())
                })?;
                let service = ConfigService::new(db);
                service.import(&config).await?;
                println!("{}", t!("config_imported", file = args.file.display()));
            }
        }
        Ok(())
    }
}
