use modql::field::Fields;
use serde::Serialize;
use sqlx::FromRow;

// region:    --- Activity Types
#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct Sport {
    pub id: i64,
    pub title: String,
    pub category: String,
}

// endregion:    --- Activity Types
