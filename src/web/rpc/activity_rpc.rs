use crate::ctx::Ctx;
use crate::model::activity::{Activity, ActivityBmc, ActivityForCreate, ActivityForUpdate};
use crate::model::ModelManager;
use crate::web::rpc::{
    ParamsForCreate, ParamsForUpdate, ParamsForUpdateMongo, ParamsIded, ParamsIdedMongo,
};
use crate::web::Result;
use hmac::digest::typenum::Mod;

pub async fn create_activity(
    ctx: Ctx,
    mm: ModelManager,
    params: ParamsForCreate<ActivityForCreate>,
) -> Result<Activity> {
    let ParamsForCreate { data } = params;

    let id = ActivityBmc::create(&ctx, &mm, data).await?;
    let activity = ActivityBmc::get(&ctx, &mm, &id).await?;

    Ok(activity)
}

pub async fn get_activity(ctx: Ctx, mm: ModelManager, params: ParamsIdedMongo) -> Result<Activity> {
    let ParamsIdedMongo { id } = params;

    let activity = ActivityBmc::get(&ctx, &mm, &id).await?;

    Ok(activity)
}

pub async fn list_activities(ctx: Ctx, mm: ModelManager) -> Result<Vec<Activity>> {
    let activities = ActivityBmc::list(&ctx, &mm).await?;

    Ok(activities)
}

pub async fn update_activity(
    ctx: Ctx,
    mm: ModelManager,
    params: ParamsForUpdateMongo<ActivityForUpdate>,
) -> Result<Activity> {
    let ParamsForUpdateMongo { id, data } = params;

    ActivityBmc::update(&ctx, &mm, &id, data).await?;

    let activity = ActivityBmc::get(&ctx, &mm, &id).await?;

    Ok(activity)
}

pub async fn delete_activity(
    ctx: Ctx,
    mm: ModelManager,
    params: ParamsIdedMongo,
) -> Result<Activity> {
    let ParamsIdedMongo { id } = params;

    let activity = ActivityBmc::get(&ctx, &mm, &id).await?;
    ActivityBmc::delete(&ctx, &mm, &id).await?;

    Ok(activity)
}
