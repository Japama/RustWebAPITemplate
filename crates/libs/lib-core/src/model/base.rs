use crate::ctx::Ctx;
use crate::model::ModelManager;
use crate::model::{Error, Result};
use bson::oid::ObjectId;
use bson::{doc, Document};
use futures::stream::TryStreamExt;
use lib_utils::time::now_utc;
use modql::field::{Field, Fields, HasFields};
use modql::filter::{FilterGroups, ListOptions};
use modql::SIden;
use mongodb::{Collection, Cursor};
use sea_query::{Condition, Expr, Iden, IntoIden, PostgresQueryBuilder, Query, TableRef};
use sea_query_binder::SqlxBinder;
use serde_json::{json, Value};
use sqlx::postgres::PgRow;
use sqlx::FromRow;

// region: Postgres

const LIST_LIMIT_DEFAULT: i64 = 1000;
const LIST_LIMIT_MAX: i64 = 5000;

#[derive(Iden)]
pub enum CommonIden {
    Id,
}

#[derive(Iden)]
pub enum TimestampIden {
    Cid,
    Ctime,
    Mid,
    Mtime,
}

pub trait PostgresDbBmc {
    const TABLE: &'static str;

    fn table_ref() -> TableRef {
        TableRef::Table(SIden(Self::TABLE).into_iden())
    }
}

pub fn compute_list_options(list_options: Option<ListOptions>) -> Result<ListOptions> {
    // When Some, validate limit
    if let Some(mut list_options) = list_options {
        // Validate the limit.
        if let Some(limit) = list_options.limit {
            if limit > LIST_LIMIT_MAX {
                return Err(Error::ListLimitOverMax {
                    max: LIST_LIMIT_MAX,
                    actual: limit,
                });
            }
        }
        // Set the default limit if no limit
        else {
            list_options.limit = Some(LIST_LIMIT_DEFAULT);
        }
        Ok(list_options)
    }
    // When None, return default
    else {
        Ok(ListOptions {
            limit: Some(LIST_LIMIT_DEFAULT),
            offset: None,
            order_bys: Some("id".into()),
        })
    }
}

pub async fn create<MC, E>(ctx: &Ctx, mm: &ModelManager, data: E) -> Result<i64>
where
    MC: PostgresDbBmc,
    E: HasFields,
{
    let db = mm.postgres_db();

    // -- Extract fields (name / sea-query value expression)
    let mut fields = data.not_none_fields();
    add_timestamps_for_create(&mut fields, ctx.user_id());
    let (columns, sea_values) = fields.for_sea_insert();

    // -- Build query
    let mut query = Query::insert();
    query
        .into_table(MC::table_ref())
        .columns(columns)
        .values(sea_values)?
        .returning(Query::returning().columns([CommonIden::Id]));

    // -- Exec query
    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let (id,) = sqlx::query_as_with::<_, (i64,), _>(&sql, values)
        .fetch_one(db)
        .await?;

    Ok(id)
}

pub async fn get<MC, E>(_ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<E>
where
    MC: PostgresDbBmc,
    E: for<'r> FromRow<'r, PgRow> + Unpin + Send,
    E: HasFields,
{
    let db = mm.postgres_db();

    // -- Build query
    let mut query = Query::select();
    query
        .from(MC::table_ref())
        .columns(E::field_column_refs())
        .and_where(Expr::col(CommonIden::Id).eq(id));

    // -- Exec query
    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let entity = sqlx::query_as_with::<_, E, _>(&sql, values)
        .fetch_optional(db)
        .await?
        .ok_or(Error::EntityNotFound {
            entity: MC::TABLE,
            id,
        })?;

    Ok(entity)
}

pub async fn list<MC, E, F>(
    _ctx: &Ctx,
    mm: &ModelManager,
    filter: Option<F>,
    list_options: Option<ListOptions>,
) -> Result<Vec<E>>
where
    MC: PostgresDbBmc,
    F: Into<FilterGroups>,
    E: for<'r> FromRow<'r, PgRow> + Unpin + Send,
    E: HasFields,
{
    let db = mm.postgres_db();

    // -- Build query
    let mut query = Query::select();
    query.from(MC::table_ref()).columns(E::field_column_refs());

    // condition from filter
    if let Some(filter) = filter {
        let filters: FilterGroups = filter.into();
        let cond: Condition = filters.try_into()?;
        query.cond_where(cond);
    }

    // list options
    let list_options = compute_list_options(list_options)?;
    list_options.apply_to_sea_query(&mut query);

    // -- Exec query
    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let entities = sqlx::query_as_with::<_, E, _>(&sql, values)
        .fetch_all(db)
        .await?;

    Ok(entities)
}

pub async fn update<MC, E>(ctx: &Ctx, mm: &ModelManager, id: i64, data: E) -> Result<()>
where
    MC: PostgresDbBmc,
    E: HasFields,
{
    let db = mm.postgres_db();

    // -- Prep data
    let mut fields = data.not_none_fields();
    add_timestamps_for_update(&mut fields, ctx.user_id());
    let fields = fields.for_sea_update();

    // -- Build query
    let mut query = Query::update();
    query
        .table(MC::table_ref())
        .values(fields)
        .and_where(Expr::col(CommonIden::Id).eq(id));

    // -- Exec query
    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let count = sqlx::query_with(&sql, values)
        .execute(db)
        .await?
        .rows_affected();

    // -- Check result
    if count == 0 {
        Err(Error::EntityNotFound {
            entity: MC::TABLE,
            id,
        })
    } else {
        Ok(())
    }
}

pub async fn delete<MC>(_ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<()>
where
    MC: PostgresDbBmc,
{
    let db = mm.postgres_db();

    // -- Build query
    let mut query = Query::delete();
    query
        .from_table(MC::table_ref())
        .and_where(Expr::col(CommonIden::Id).eq(id));

    // -- Exec query
    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let count = sqlx::query_with(&sql, values)
        .execute(db)
        .await?
        .rows_affected();

    // -- Check result
    if count == 0 {
        Err(Error::EntityNotFound {
            entity: MC::TABLE,
            id,
        })
    } else {
        Ok(())
    }
}

// endregion: Postgres

// region: MongoDB

pub trait MongoDbBmc {
    const DATABASE: &'static str;
    const COLLECTION: &'static str;
}

pub async fn create_mongo<MC>(_ctx: &Ctx, mm: &ModelManager, data: Value) -> Result<String>
where
    MC: MongoDbBmc,
{
    let db = mm.mongo_db().database(MC::DATABASE);
    let collection = db.collection(MC::COLLECTION);
    match collection.insert_one(data, None).await {
        Ok(e) => {
            if let Some(oid) = e.inserted_id.as_object_id() {
                let id_str = oid.to_hex();
                return Ok(id_str);
            } else {
                Err(Error::MongoQueryError(
                    "Fail getting id from ObjectId".to_string(),
                ))
            }
        }
        Err(e) => Err(Error::MongoQueryError(e.to_string())),
    }
}

pub async fn get_mongo<MC>(_ctx: &Ctx, mm: &ModelManager, oid: ObjectId) -> Result<Value>
where
    MC: MongoDbBmc,
{
    let db = mm.mongo_db().database(MC::DATABASE);
    let collection = db.collection(MC::COLLECTION);

    let filter = doc! {"_id": oid};

    match collection.find_one(filter, None).await {
        Ok(result) => match result {
            Some(document) => Ok(document),
            None => Err(Error::MongoEntityNotFound {
                entity: MC::COLLECTION,
                id: oid.to_string(),
            }),
        },
        Err(e) => Err(Error::MongoQueryError(e.to_string())),
    }
}

pub async fn list_mongo<MC>(_ctx: &Ctx, mm: &ModelManager) -> Result<Value>
where
    MC: MongoDbBmc,
{
    let db = mm.mongo_db().database(MC::DATABASE);
    let collection = db.collection(MC::COLLECTION);

    // Puedes agregar más opciones aquí para personalizar tu consulta, como filtrar o ordenar.

    let mut cursor: Cursor<Document> = collection.find(None, None).await?;

    let mut documents: Vec<Document> = Vec::new();

    while let Some(doc) = cursor.try_next().await? {
        documents.push(doc);
    }
    Ok(json!(documents))
}

pub async fn update_mongo<MC>(
    _ctx: &Ctx,
    mm: &ModelManager,
    oid: ObjectId,
    data: Value,
) -> Result<()>
where
    MC: MongoDbBmc,
{
    let db = mm.mongo_db().database(MC::DATABASE);
    let collection: Collection<Value> = db.collection(MC::COLLECTION);

    let filter = doc! {"_id": oid};
    let update_bson = bson::to_bson(&data).map_err(|e| Error::MongoQueryError(e.to_string()))?;
    let update_doc = doc! {"$set": update_bson};

    match collection.update_one(filter, update_doc, None).await {
        Ok(result) => {
            if result.modified_count == 1 {
                Ok(())
            } else {
                Err(Error::MongoEntityNotFound {
                    entity: MC::COLLECTION,
                    id: oid.to_string(),
                })
            }
        }
        Err(e) => Err(Error::MongoQueryError(e.to_string())),
    }
}

pub async fn delete_mongo<MC>(_ctx: &Ctx, mm: &ModelManager, oid: ObjectId) -> Result<()>
where
    MC: MongoDbBmc,
{
    let db = mm.mongo_db().database(MC::DATABASE);
    let collection: Collection<Value> = db.collection(MC::COLLECTION);

    let filter = doc! {"_id": oid};

    match collection.delete_one(filter, None).await {
        Ok(result) => {
            if result.deleted_count == 1 {
                Ok(())
            } else {
                Err(Error::MongoEntityNotFound {
                    entity: MC::COLLECTION,
                    id: oid.to_string(),
                })
            }
        }
        Err(e) => Err(Error::MongoQueryError(e.to_string())),
    }
}

// endregion: MongoDB

// region:    --- Utils

/// Update the timestamps info for create
/// (e.g., cid, ctime, and mid, mtime will be updated with the same values)
pub fn add_timestamps_for_create(fields: &mut Fields, user_id: i64) {
    let now = now_utc();
    fields.push(Field::new(TimestampIden::Cid.into_iden(), user_id.into()));
    fields.push(Field::new(TimestampIden::Ctime.into_iden(), now.into()));

    fields.push(Field::new(TimestampIden::Mid.into_iden(), user_id.into()));
    fields.push(Field::new(TimestampIden::Mtime.into_iden(), now.into()));
}

/// Update the timestamps info only for update.
/// (.e.g., only mid, mtime will be udpated)
pub fn add_timestamps_for_update(fields: &mut Fields, user_id: i64) {
    let now = now_utc();
    fields.push(Field::new(TimestampIden::Mid.into_iden(), user_id.into()));
    fields.push(Field::new(TimestampIden::Mtime.into_iden(), now.into()));
}

// endregion: --- Utils
