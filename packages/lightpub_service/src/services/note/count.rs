use crate::{
    ServiceResult,
    services::{MapToUnknown, db::MaybeTxConn},
};
use sea_orm::{Condition, prelude::*};

pub async fn count_local_notes(tx: &MaybeTxConn) -> ServiceResult<u64> {
    let count = entity::note::Entity::find()
        .find_also_related(entity::user::Entity)
        .filter(
            Condition::all()
                .add(entity::note::Column::DeletedAt.is_null())
                .add(entity::note::Column::RenoteOfId.is_null())
                .add(entity::user::Column::Domain.eq("")),
        )
        .count(tx)
        .await
        .map_err_unknown()?;

    Ok(count)
}
