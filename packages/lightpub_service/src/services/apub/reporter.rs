use crate::services::db::MaybeTxConn;
use crate::{ServiceResult, services::MapToUnknown};
use chrono::Utc;
use sea_orm::ActiveModelTrait;
use sea_orm::Set;

pub async fn report_apub_error(
    tx: &MaybeTxConn,
    payload: impl Into<String>,
    error_msg: impl Into<String>,
) -> ServiceResult<()> {
    let model = entity::apub_error_report::ActiveModel {
        activity: Set(payload.into()),
        error_msg: Set(error_msg.into()),
        received_at: Set(Utc::now().naive_utc()),
        ..Default::default()
    };
    model.save(tx).await.map_err_unknown()?;

    Ok(())
}
