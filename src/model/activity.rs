use crate::ctx::Ctx;
use crate::model::base::{self, PostgresDbBmc};
use crate::model::task::TaskForCreate;
use crate::model::Error::{MongoDuplicateError, MongoQueryError};
use crate::model::ModelManager;
use crate::model::{Error, Result};
use bson::Document;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlb::Fields;
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Category {
    Senior,
    Sub23,
    Sub20,
    Sub18,
    Sub16,
    Sub14,
    Sub12,
    Sub10,
    Sub8,
    Sub6,
}

const DATABASE: &str = "sportsGuide";
const COLLECTION: &str = "activities";

#[derive(Debug)]
struct Activity {
    id: i32,
    name: String,
    sport_id: i32,
    category: Category,
    description: String,
    multimedia_links: Vec<String>,
    rating: f32,
    tags: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ActivityForCreate {
    name: String,
    sport_id: i32,
    category: Category,
    description: String,
    multimedia_links: Vec<String>,
    rating: f32,
    tags: Vec<String>,
}

#[derive(Fields, Deserialize)]
pub struct ActivityForUpdate {
    pub title: Option<String>,
}

pub struct ActivityBmc;

impl crate::model::activity::ActivityBmc {
    pub async fn create(
        ctx: &Ctx,
        mm: &ModelManager,
        activity_c: ActivityForCreate,
    ) -> Result<String> {
        let db = mm.mongo_db.database(DATABASE);
        let collection = db.collection(COLLECTION);

        let activity = get_json(activity_c);
        match collection.insert_one(activity, None).await {
            Ok(_) => Ok("Inserción exitosa".to_string()),
            Err(e) => {
                if e.to_string()
                    .contains("E11000 duplicate key error collection")
                {
                    Err(Error::MongoDuplicateError {})
                } else {
                    Err(Error::MongoQueryError {})
                }
            }
        }
    }
}

// endregion:    --- Activity Types

fn get_json(activity_c: ActivityForCreate) -> Value {
    let activity = json!({
        "name": activity_c.name,
        "sport_id": activity_c.sport_id,
        "category": activity_c.category,
        "description": activity_c.description,
        "multimedia_links": activity_c.multimedia_links,
        "rating": activity_c.rating,
        "tags": activity_c.tags,
    });
    activity
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::_dev_utils;
    use crate::model::base::create;
    use anyhow::Result;
    use serial_test::serial;

    #[serial]
    #[tokio::test]
    async fn test_create_activity_ok() -> Result<()> {
        // -- Setup & Fixtures
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_ctx();
        let fx_title = "test_create_ok title";

        // -- Exec
        let activity = ActivityForCreate {
            name: "Ejemplo de Actividad".to_string(),
            sport_id: 123,
            category: Category::Senior,
            description: "Esta es una actividad de muestra.".to_string(),
            multimedia_links: vec![
                "https://ejemplo.com/imagen1.jpg".to_string(),
                "https://ejemplo.com/video.mp4".to_string(),
            ],
            rating: 4.5,
            tags: vec!["deporte".to_string(), "aire libre".to_string()],
        };
        let id = ActivityBmc::create(&ctx, &mm, activity).await?;

        // -- Check
        // let task = TaskBmc::get(&ctx, &mm, id).await?;
        // assert_eq!(task.title, fx_title);

        // -- Clean
        // TaskBmc::delete(&ctx, &mm, id).await?;

        Ok(())
    }
}
