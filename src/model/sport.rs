use crate::ctx::Ctx;
use crate::model::base::{self, PostgresDbBmc};
use crate::model::ModelManager;
use crate::model::{Error, Result};
use serde::{Deserialize, Serialize};
use sqlb::Fields;
use sqlx::FromRow;

// region:    --- Activity Types
#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct Sport {
    pub id: i64,
    pub title: String,
    pub category: String,
}

// endregion:    --- Activity Types
