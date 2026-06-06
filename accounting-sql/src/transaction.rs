use crate::repo::account::AccountRepo;
use crate::repo::attachment::AttachmentRepo;
use crate::repo::channel::ChannelRepo;
use crate::repo::commodity::CommodityRepo;
use crate::repo::member::MemberRepo;
use crate::repo::posting::PostingRepo;
use crate::repo::tag::TagRepo;
use crate::repo::transaction::TransactionRepo;

/// 事务 trait，包含所有 Repository 与提交能力
#[allow(async_fn_in_trait)]
pub trait Transaction: Send {
    /// 获取 AccountRepo
    fn account_repo(&self) -> &dyn AccountRepo;
    /// 获取 CommodityRepo
    fn commodity_repo(&self) -> &dyn CommodityRepo;
    /// 获取 MemberRepo
    fn member_repo(&self) -> &dyn MemberRepo;
    /// 获取 ChannelRepo
    fn channel_repo(&self) -> &dyn ChannelRepo;
    /// 获取 TagRepo
    fn tag_repo(&self) -> &dyn TagRepo;
    /// 获取 AttachmentRepo
    fn attachment_repo(&self) -> &dyn AttachmentRepo;
    /// 获取 TransactionRepo
    fn transaction_repo(&self) -> &dyn TransactionRepo;
    /// 获取 PostingRepo
    fn posting_repo(&self) -> &dyn PostingRepo;

    /// 提交事务
    async fn commit(self) -> Result<(), crate::error::DbError>;
}
