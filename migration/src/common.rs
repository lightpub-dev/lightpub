use sea_orm_migration::{
    prelude::{ColumnDef, Expr, IntoIden, SimpleExpr},
    sea_orm::Iden,
};

pub const URL_LENGTH: u32 = 512;
pub const DATETIME_6: &str = "DATETIME(6)";
pub const CURRENT_TIMESTAMP_6: &str = "CURRENT_TIMESTAMP(6)";

enum AdditionalMysqlTypes {
    Datetime6,
}

impl Iden for AdditionalMysqlTypes {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        let ty = match self {
            Self::Datetime6 => "datetime(6)",
        };
        write!(s, "{ty}").unwrap()
    }
}

pub fn current_timestamp_6() -> SimpleExpr {
    Expr::cust(CURRENT_TIMESTAMP_6)
}

pub fn datetime_6(name: impl IntoIden) -> ColumnDef {
    ColumnDef::new(name)
        .custom(AdditionalMysqlTypes::Datetime6)
        .not_null()
        .take()
}

pub fn datetime_6_null(name: impl IntoIden) -> ColumnDef {
    ColumnDef::new(name)
        .custom(AdditionalMysqlTypes::Datetime6)
        .null()
        .take()
}
