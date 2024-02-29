pub use sea_orm_migration::prelude::*;

pub mod ident;
mod m20220101_000001_create_table;
mod m20240229_084204_user_follow;
mod m20240229_085315_usertoken;
mod m20240229_085759_post;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20240229_084204_user_follow::Migration),
            Box::new(m20240229_085315_usertoken::Migration),
            Box::new(m20240229_085759_post::Migration),
        ]
    }
}
