//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.7

use super::sea_orm_active_enums::Visibility;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "note")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Binary(16)")]
    pub id: Vec<u8>,
    pub url: Option<String>,
    pub view_url: Option<String>,
    #[sea_orm(column_type = "Binary(16)")]
    pub author_id: Vec<u8>,
    #[sea_orm(column_type = "Text", nullable)]
    pub content: Option<String>,
    pub content_type: Option<String>,
    pub created_at: DateTime,
    pub inserted_at: DateTime,
    pub updated_at: Option<DateTime>,
    pub deleted_at: Option<DateTime>,
    pub visibility: Visibility,
    #[sea_orm(column_type = "Binary(16)", nullable)]
    pub reply_to_id: Option<Vec<u8>>,
    #[sea_orm(column_type = "Binary(16)", nullable)]
    pub renote_of_id: Option<Vec<u8>>,
    pub sensitive: i8,
    pub fetched_at: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::note_like::Entity")]
    NoteLike,
    #[sea_orm(has_many = "super::note_mention::Entity")]
    NoteMention,
    #[sea_orm(has_many = "super::note_tag::Entity")]
    NoteTag,
    #[sea_orm(has_many = "super::note_upload::Entity")]
    NoteUpload,
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::AuthorId",
        to = "super::user::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    User,
}

impl Related<super::note_like::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::NoteLike.def()
    }
}

impl Related<super::note_mention::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::NoteMention.def()
    }
}

impl Related<super::note_tag::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::NoteTag.def()
    }
}

impl Related<super::note_upload::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::NoteUpload.def()
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
