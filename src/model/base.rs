use crate::ctx::Ctx;
use crate::model::Error::MongoInvalidIDError;
use crate::model::ModelManager;
use crate::model::{Error, Result};
use axum::Json;
use bson::oid::ObjectId;
use bson::{doc, Document};
use futures::stream::TryStreamExt;
use mongodb::{Collection, Cursor};
use serde::Serialize;
use serde_json::{json, Value};
use sqlb::HasFields;
use sqlx::postgres::PgRow;
use sqlx::FromRow;
use std::str::FromStr;
use tracing::info;

// region: Postgres

pub trait PostgresDbBmc {
    const TABLE: &'static str;
}

pub async fn create<MC, E>(_ctx: &Ctx, mm: &ModelManager, data: E) -> Result<i64>
where
    MC: PostgresDbBmc,
    E: HasFields,
{
    let db = mm.postgres_db();

    let fields = data.not_none_fields();
    let (id,) = sqlb::insert()
        .table(MC::TABLE)
        .data(fields)
        .returning(&["id"])
        .fetch_one::<_, (i64,)>(db)
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

    let entity: E = sqlb::select()
        .table(MC::TABLE)
        .columns(E::field_names())
        .and_where("id", "=", id)
        .fetch_optional(db)
        .await?
        .ok_or(Error::EntityNotFound {
            entity: MC::TABLE,
            id,
        })?;

    Ok(entity)
}

pub async fn list<MC, E>(_ctx: &Ctx, mm: &ModelManager) -> Result<Vec<E>>
where
    MC: PostgresDbBmc,
    E: for<'r> FromRow<'r, PgRow> + Unpin + Send,
    E: HasFields,
{
    let db = mm.postgres_db();

    let entities: Vec<E> = sqlb::select()
        .table(MC::TABLE)
        .columns(E::field_names())
        .order_by("id")
        .fetch_all(db)
        .await?;

    Ok(entities)
}

pub async fn update<MC, E>(_ctx: &Ctx, mm: &ModelManager, id: i64, data: E) -> Result<()>
where
    MC: PostgresDbBmc,
    E: HasFields,
{
    let db = mm.postgres_db();

    let fields = data.not_none_fields();
    let count = sqlb::update()
        .table(MC::TABLE)
        .and_where("id", "=", id)
        .data(fields)
        .exec(db)
        .await?;

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

    let count = sqlb::delete()
        .table(MC::TABLE)
        .and_where("id", "=", id)
        .exec(db)
        .await?;

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

    let mut documents = Vec::new();

    while let Some(doc) = cursor.try_next().await? {
        documents.push(doc);
    }
    Ok(json!(documents))
}

pub async fn update_mongo<MC>(
    ctx: &Ctx,
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

pub async fn delete_mongo<MC>(ctx: &Ctx, mm: &ModelManager, oid: ObjectId) -> Result<()>
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
