use crate::ctx::Ctx;
use crate::model::base::MongoDbBmc;
use crate::model::Error::{MongoInvalidIDError, MongoQueryError};
use crate::model::{base, ModelManager};
use crate::model::{Error, Result};
use bson::oid::ObjectId;
use bson::Document;
use serde::{Deserialize, Serialize};
use serde_json::{from_value, json, to_value, Value};
use std::str::FromStr;

#[derive(Clone, Serialize, Deserialize)]
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

// region: Activity types

#[derive(Serialize, Deserialize)]
pub struct Activity {
    _id: ObjectId,
    name: String,
    sport_id: i32,
    category: Category,
    description: String,
    multimedia_links: Vec<String>,
    rating: f32,
    tags: Vec<String>,
    user_id: i64,
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
    user_id: i64,
}

#[derive(Serialize, Deserialize)]
pub struct ActivityForUpdate {
    category: Category,
    description: String,
    multimedia_links: Vec<String>,
    rating: f32,
    tags: Vec<String>,
}

// endregion: Activity types

pub struct ActivityBmc;

impl MongoDbBmc for ActivityBmc {
    const DATABASE: &'static str = "sportsGuide";
    const COLLECTION: &'static str = "activities";
}

impl ActivityBmc {
    pub async fn create(
        ctx: &Ctx,
        mm: &ModelManager,
        activity_c: ActivityForCreate,
    ) -> Result<String> {
        let activity = to_value(activity_c).unwrap();
        let id: String = base::create_mongo::<Self>(ctx, mm, activity).await?;
        Ok(id)
    }

    pub async fn get(ctx: &Ctx, mm: &ModelManager, id: &str) -> Result<Activity> {
        let oid = ObjectId::from_str(&id).map_err(|_| MongoInvalidIDError(id.to_owned()))?;
        let value: Value = base::get_mongo::<Self>(ctx, mm, oid).await?;
        let activity: Activity = from_value(value).unwrap();
        Ok(activity)
    }

    pub async fn list(ctx: &Ctx, mm: &ModelManager) -> Result<Vec<Activity>> {
        let value: Value = base::list_mongo::<Self>(ctx, mm).await?;
        let activities: Vec<Activity> =
            serde_json::from_value(value).map_err(|err| MongoQueryError(err.to_string()))?;
        Ok(activities)
    }

    pub async fn update(
        ctx: &Ctx,
        mm: &ModelManager,
        id: &str,
        activity_u: ActivityForUpdate,
    ) -> Result<()> {
        let oid = ObjectId::from_str(&id).map_err(|_| MongoInvalidIDError(id.to_string()))?;
        let activity = to_value(activity_u).unwrap();
        base::update_mongo::<Self>(ctx, mm, oid, activity).await
    }

    pub async fn delete(ctx: &Ctx, mm: &ModelManager, id: &str) -> Result<()> {
        let oid = ObjectId::from_str(&id).map_err(|_| MongoInvalidIDError(id.to_string()))?;
        base::delete_mongo::<Self>(ctx, mm, oid).await
    }
}

// region: Tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::_dev_utils;
    use crate::model::base::create;
    use crate::model::task::{Task, TaskBmc};
    use anyhow::Result;
    use bson::Uuid;
    use serial_test::serial;
    use std::thread::sleep;
    use std::time::Duration;

    #[serial]
    #[tokio::test]
    async fn test_create_activity_ok() -> Result<()> {
        // -- Setup & Fixtures
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_ctx();
        let name = "Ejemplo de Actividad".to_string();
        let activity_c = ActivityForCreate {
            name: name.clone(),
            sport_id: 123,
            category: Category::Senior,
            description: "Esta es una actividad de muestra.".to_string(),
            multimedia_links: vec![
                "https://ejemplo.com/imagen1.jpg".to_string(),
                "https://ejemplo.com/video.mp4".to_string(),
            ],
            rating: 4.5,
            tags: vec!["deporte".to_string(), "aire libre".to_string()],
            user_id: ctx.user_id(),
        };

        // -- Exec
        let id = ActivityBmc::create(&ctx, &mm, activity_c).await?;

        // -- Check
        let activity = ActivityBmc::get(&ctx, &mm, &id).await?;
        assert_eq!(activity.name, name);

        // -- Clean
        ActivityBmc::delete(&ctx, &mm, &id).await?;

        Ok(())
    }

    #[serial]
    #[tokio::test]
    async fn test_get_activity_ok() -> Result<()> {
        // -- Setup & Fixtures
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_ctx();
        let name = "Ejemplo de Actividad".to_string();
        let activity_c = ActivityForCreate {
            name: name.clone(),
            sport_id: 123,
            category: Category::Senior,
            description: "Esta es una actividad de muestra.".to_string(),
            multimedia_links: vec![
                "https://ejemplo.com/imagen1.jpg".to_string(),
                "https://ejemplo.com/video.mp4".to_string(),
            ],
            rating: 4.5,
            tags: vec!["deporte".to_string(), "aire libre".to_string()],
            user_id: ctx.user_id(),
        };
        let id = ActivityBmc::create(&ctx, &mm, activity_c).await?;

        // -- Exec
        let activity = ActivityBmc::get(&ctx, &mm, &id).await?;

        // -- Check
        assert_eq!(activity.name, name);

        // -- Clean
        ActivityBmc::delete(&ctx, &mm, &id).await?;

        Ok(())
    }

    #[serial]
    #[tokio::test]
    async fn test_list_activities_ok() -> Result<()> {
        // -- Setup & Fixtures
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_ctx();

        let name1 = "Ejemplo de Actividad - 1".to_string();
        let name2 = "Ejemplo de Actividad - 2".to_string();

        let activity_c1 = ActivityForCreate {
            name: name1.clone(),
            sport_id: 123,
            category: Category::Senior,
            description: "Esta es una actividad de muestra.".to_string(),
            multimedia_links: vec![
                "https://ejemplo.com/imagen1.jpg".to_string(),
                "https://ejemplo.com/video.mp4".to_string(),
            ],
            rating: 4.5,
            tags: vec!["deporte".to_string(), "aire libre".to_string()],
            user_id: ctx.user_id(),
        };

        let activity_c2 = ActivityForCreate {
            name: name2.clone(),
            sport_id: 123,
            category: Category::Senior,
            description: "Esta es una actividad de muestra.".to_string(),
            multimedia_links: vec![
                "https://ejemplo.com/imagen1.jpg".to_string(),
                "https://ejemplo.com/video.mp4".to_string(),
            ],
            rating: 4.5,
            tags: vec!["deporte".to_string(), "aire libre".to_string()],
            user_id: ctx.user_id(),
        };

        let id1 = ActivityBmc::create(&ctx, &mm, activity_c1).await?;
        let id2 = ActivityBmc::create(&ctx, &mm, activity_c2).await?;

        // -- Exec
        let activities = ActivityBmc::list(&ctx, &mm).await?;

        // -- Check
        assert_eq!(activities.len(), 2, "number of seeded activities.");

        // -- Clean
        for activity in activities {
            ActivityBmc::delete(&ctx, &mm, &activity._id.to_hex()).await?;
        }

        Ok(())
    }
    #[serial]
    #[tokio::test]
    async fn test_update_activity_ok() -> Result<()> {
        // -- Setup & Fixtures
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_ctx();

        let desc1 = "Ejemplo de Actividad - 1".to_string();
        let activity_c = ActivityForCreate {
            name: "Actividad".to_string(),
            sport_id: 123,
            category: Category::Senior,
            description: desc1.clone(),
            multimedia_links: vec![
                "https://ejemplo.com/imagen1.jpg".to_string(),
                "https://ejemplo.com/video.mp4".to_string(),
            ],
            rating: 4.5,
            tags: vec!["deporte".to_string(), "aire libre".to_string()],
            user_id: ctx.user_id(),
        };

        let id = ActivityBmc::create(&ctx, &mm, activity_c).await?;
        let activity_init = ActivityBmc::get(&ctx, &mm, &id).await?;
        assert_eq!(desc1, activity_init.description);

        let rand = Uuid::new().to_string();
        let desc2 = "Ejemplo de Actividad - ".to_string() + &rand;
        let activity_u = ActivityForUpdate {
            category: Category::Sub10,
            description: desc2.clone(),
            multimedia_links: activity_init.multimedia_links,
            rating: activity_init.rating,
            tags: activity_init.tags,
        };

        // -- Exec
        let activity = ActivityBmc::update(&ctx, &mm, &id, activity_u).await?;
        let activity = ActivityBmc::get(&ctx, &mm, &id).await?;

        // -- Check
        assert_eq!(activity.description, desc2, "Description change");

        // -- Clean
        ActivityBmc::delete(&ctx, &mm, &activity._id.to_hex()).await?;

        Ok(())
    }
    #[serial]
    #[tokio::test]
    async fn test_delete_activity_err_not_found() -> Result<()> {
        // -- Setup & Fixtures
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_ctx();
        let id = "65078008c1318fdcd4797c91".to_string();

        // -- Exec
        let res = ActivityBmc::delete(&ctx, &mm, &id).await;

        // -- Check
        assert!(
            matches!(
                res,
                Err(Error::MongoEntityNotFound {
                    entity: "activities",
                    id
                })
            ),
            "EntityNotFound not matching"
        );

        Ok(())
    }
}
// endregion: Tests
