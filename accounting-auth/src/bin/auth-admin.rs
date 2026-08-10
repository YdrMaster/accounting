//! auth-admin：用户管理命令行工具（公网不开注册/改密接口，见归档 `add-user-auth` design.md D8）

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "auth-admin", about = "accounting-auth 用户管理工具")]
struct Cli {
    /// 认证数据库文件路径
    #[arg(long, default_value = "auth.db", global = true)]
    db: String,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// 用户管理
    User {
        #[command(subcommand)]
        action: UserAction,
    },
}

#[derive(Subcommand)]
enum UserAction {
    /// 创建用户
    Add {
        /// 用户名（唯一）
        #[arg(long)]
        username: String,
        /// 密码
        #[arg(long)]
        password: String,
        /// 显示名（默认同用户名）
        #[arg(long)]
        display_name: Option<String>,
    },
    /// 修改密码
    Passwd {
        /// 用户名
        #[arg(long)]
        username: String,
        /// 新密码
        #[arg(long)]
        password: String,
    },
    /// 列出全部用户
    List,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(cli).await {
        eprintln!("错误: {e}");
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> accounting_auth::Result<()> {
    let state = accounting_auth::init(&cli.db).await?;
    let db = state.db();

    match cli.command {
        Command::User { action } => match action {
            UserAction::Add {
                username,
                password,
                display_name,
            } => {
                let hash = accounting_auth::service::password::hash_password(&password)?;
                let display_name = display_name.unwrap_or_else(|| username.clone());
                let id = db.create_user(&username, &hash, &display_name).await?;
                println!("已创建用户 {username}（id={id}，display_name={display_name}）");
            }
            UserAction::Passwd { username, password } => {
                let hash = accounting_auth::service::password::hash_password(&password)?;
                db.update_password(&username, &hash).await?;
                println!("已更新用户 {username} 的密码");
            }
            UserAction::List => {
                let users = db.list_users().await?;
                if users.is_empty() {
                    println!("（无用户）");
                } else {
                    println!("{:<6}{:<20}{:<20}TOTP", "ID", "USERNAME", "DISPLAY_NAME");
                    for u in users {
                        println!(
                            "{:<6}{:<20}{:<20}{}",
                            u.id,
                            u.username,
                            u.display_name,
                            if u.totp_enabled {
                                "已开启"
                            } else {
                                "未开启"
                            }
                        );
                    }
                }
            }
        },
    }
    Ok(())
}
