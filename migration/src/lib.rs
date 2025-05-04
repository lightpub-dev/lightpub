pub use sea_orm_migration::prelude::*;

pub mod common;
mod m20220101_000001_create_table;
mod m20250202_050205_notes;
mod m20250202_085027_follower;
mod m20250202_093800_note_relation;
mod m20250202_094153_follow_relation;
mod m20250204_154802_user_unique;
mod m20250209_163317_bookmark;
mod m20250210_064038_notification;
mod m20250210_064527_notification_fk;
mod m20250211_023215_hashtag;
mod m20250211_025745_mention;
mod m20250211_132721_uploads;
mod m20250212_021856_upload_mime;
mod m20250213_040400_user_avatar_fk;
mod m20250215_044502_optimize;
mod m20250215_045214_trending;
mod m20250216_033430_user_url_unique;
mod m20250216_033755_public_keys;
mod m20250216_143035_note_fetched_at;
mod m20250218_060414_user_created_at_optional;
mod m20250219_030944_optimize;
mod m20250219_131352_target_inbox;
mod m20250220_110034_timeline_query;
mod m20250221_041103_timeline_fix;
mod m20250225_173949_blocking;
mod m20250310_160527_apub_error_report;
mod m20250316_060953_timeline_procedure;
mod m20250319_015119_push_notification;
mod m20250504_005109_totp;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20250202_050205_notes::Migration),
            Box::new(m20250202_085027_follower::Migration),
            Box::new(m20250202_093800_note_relation::Migration),
            Box::new(m20250202_094153_follow_relation::Migration),
            Box::new(m20250204_154802_user_unique::Migration),
            Box::new(m20250209_163317_bookmark::Migration),
            Box::new(m20250210_064038_notification::Migration),
            Box::new(m20250210_064527_notification_fk::Migration),
            Box::new(m20250211_023215_hashtag::Migration),
            Box::new(m20250211_025745_mention::Migration),
            Box::new(m20250211_132721_uploads::Migration),
            Box::new(m20250212_021856_upload_mime::Migration),
            Box::new(m20250213_040400_user_avatar_fk::Migration),
            Box::new(m20250215_044502_optimize::Migration),
            Box::new(m20250215_045214_trending::Migration),
            Box::new(m20250216_033430_user_url_unique::Migration),
            Box::new(m20250216_033755_public_keys::Migration),
            Box::new(m20250216_143035_note_fetched_at::Migration),
            Box::new(m20250218_060414_user_created_at_optional::Migration),
            Box::new(m20250219_030944_optimize::Migration),
            Box::new(m20250219_131352_target_inbox::Migration),
            Box::new(m20250220_110034_timeline_query::Migration),
            Box::new(m20250221_041103_timeline_fix::Migration),
            Box::new(m20250225_173949_blocking::Migration),
            Box::new(m20250310_160527_apub_error_report::Migration),
            Box::new(m20250316_060953_timeline_procedure::Migration),
            Box::new(m20250319_015119_push_notification::Migration),
            Box::new(m20250504_005109_totp::Migration),
        ]
    }
}
