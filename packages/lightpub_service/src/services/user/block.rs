use sea_orm::{Condition, Set};

use crate::{
    ServiceResult,
    services::{
        MapToUnknown, ServiceError,
        db::{MaybeTxConn, is_unique_constraint_error},
        id::{Identifier, UserID},
    },
};
use sea_orm::prelude::*;

pub async fn block_user(
    tx: &MaybeTxConn,
    blocker_id: UserID,
    blocked_id: UserID,
) -> ServiceResult<()> {
    let model = entity::user_block::ActiveModel {
        blocker_id: Set(blocker_id.as_db()),
        blocked_id: Set(blocked_id.as_db()),
        ..Default::default()
    };

    let result = model.insert(tx).await;
    match result {
        Ok(_) => {}
        Err(e) if is_unique_constraint_error(&e) => {
            // no further action needed
            return Ok(());
        }
        Err(e) => {
            return Err(ServiceError::unknown(e));
        }
    }

    // TODO: activitypub send

    Ok(())
}

pub async fn unblock_user(
    tx: &MaybeTxConn,
    blocker_id: UserID,
    blocked_id: UserID,
) -> ServiceResult<()> {
    let model = entity::user_block::Entity::find()
        .filter(
            Condition::all()
                .add(entity::user_block::Column::BlockerId.eq(blocker_id.as_db()))
                .add(entity::user_block::Column::BlockedId.eq(blocked_id.as_db())),
        )
        .one(tx)
        .await
        .map_err_unknown()?;

    match model {
        Some(model) => {
            model.delete(tx).await.map_err_unknown()?;

            // TODO: activitypub send
            Ok(())
        }
        None => {
            // no further action needed
            Ok(())
        }
    }
}

pub async fn is_blocking_user(
    tx: &MaybeTxConn,
    blocker_id: UserID,
    blocked_id: UserID,
) -> ServiceResult<bool> {
    let count = entity::user_block::Entity::find()
        .filter(
            Condition::all()
                .add(entity::user_block::Column::BlockerId.eq(blocker_id.as_db()))
                .add(entity::user_block::Column::BlockedId.eq(blocked_id.as_db())),
        )
        .count(tx)
        .await
        .map_err_unknown()?;

    Ok(count > 0)
}

pub async fn is_blocking_or_blocked(
    tx: &MaybeTxConn,
    user1: UserID,
    user2: UserID,
) -> ServiceResult<bool> {
    let count = entity::user_block::Entity::find()
        .filter(
            Condition::any()
                .add(
                    Condition::all()
                        .add(entity::user_block::Column::BlockerId.eq(user1.as_db()))
                        .add(entity::user_block::Column::BlockedId.eq(user2.as_db())),
                )
                .add(
                    Condition::all()
                        .add(entity::user_block::Column::BlockerId.eq(user2.as_db()))
                        .add(entity::user_block::Column::BlockedId.eq(user1.as_db())),
                ),
        )
        .count(tx)
        .await
        .map_err_unknown()?;

    Ok(count > 0)
}
