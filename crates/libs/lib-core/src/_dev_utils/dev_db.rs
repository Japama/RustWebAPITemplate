use bson::doc;
use mongodb::options::ClientOptions;
use mongodb::Client;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tracing::info;

use crate::ctx::Ctx;
use crate::model::activity::Activity;
use crate::model::user::{User, UserBmc};
use crate::model::ModelManager;

type Db = Pool<Postgres>;

// NOTE: Hardcode to prevent deployed system db update.
//  POSTGRES
const PG_DEV_POSTGRES_URL: &str = "postgres://postgres:japama@localhost/postgres";
const PG_DEV_APP_URL: &str = "postgres://postgres:japama@localhost/sports_guide";

//  MONGODB
const MDB_DEV_MONGODB_URL: &str = "mongodb://127.0.0.1:27017";
// const MDB_DEV_MONGODB_ATLAS_URL: &str =
//     "mongodb+srv://jbvc91:ICy89kEfX5PaP3ij@psg.jodkz9a.mongodb.net/?retryWrites=true&w=majority";
const MDB_DEV_MONGODB_DATABASE: &str = "sportsGuide";
const MDB_DEV_MONGODB_COLLECTION: &str = "activities";

// sql files
const SQL_RECREATE_DB_FILE_NAME: &str = "00-recreate-db.sql";
const SQL_DIR: &str = "sql/dev_initial";

const DEMO_PWD: &str = "welcome";

pub async fn init_dev_db() -> Result<(), Box<dyn std::error::Error>> {
    info!("{:<12} - init_dev_db()", "FOR-DEV-ONLY");

    // -- Get the sql_dir
    // Note: This is because cargo test and cargo run won't give the same
    //       current_dir given the worspace layout.
    let current_dir = std::env::current_dir().unwrap();
    let v: Vec<_> = current_dir.components().collect();
    let path_comp = v.get(v.len().wrapping_sub(3));
    let base_dir = if Some(true) == path_comp.map(|c| c.as_os_str() == "crates") {
        v[..v.len() - 3].iter().collect::<PathBuf>()
    } else {
        current_dir.clone()
    };
    let sql_dir = base_dir.join(SQL_DIR);

    // -- Create the app_db/app_user with the postgres user.
    {
        let sql_recreate_db_file = sql_dir.join(SQL_RECREATE_DB_FILE_NAME);
        let root_db = new_db_pool(PG_DEV_POSTGRES_URL).await?;
        pexec(&root_db, &sql_recreate_db_file).await?;
    }

    // -- Get sql files.
    let mut paths: Vec<PathBuf> = fs::read_dir(sql_dir)?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .collect();
    paths.sort();

    // -- SQL Execute each file.
    let app_db = new_db_pool(PG_DEV_APP_URL).await?;

    for path in paths {
        let path_str = path.to_string_lossy();

        if path_str.ends_with(".sql") && !path_str.ends_with(SQL_RECREATE_DB_FILE_NAME) {
            pexec(&app_db, &path).await?;
        }
    }

    // -- Init model layer.
    let mm = ModelManager::new().await?;
    let ctx = Ctx::root_ctx();

    // -- Set demo1 pwd
    let demo1_user: User = UserBmc::first_by_username(&ctx, &mm, "demo1")
        .await?
        .unwrap();
    UserBmc::update_pwd(&ctx, &mm, demo1_user.id, DEMO_PWD).await?;
    info!(
        "{:<12} - init_dev_db - set demo1 pwd",
        "FOR-DEV-ONLY"
    );

    Ok(())
}

pub async fn init_dev_mongodb() -> Result<(), Box<dyn std::error::Error>> {
    info!("{:<12} - init_dev_mongodb()", "FOR-DEV-ONLY");

    // -- Create the sports_guide/app_user_db with the postgres user
    // if let Ok(root_db) = new_mongo_client(MDB_DEV_MONGODB_ATLAS_URL).await {
    if let Ok(root_db) = new_mongo_client(MDB_DEV_MONGODB_URL).await {
        // // Eliminar la base de datos
        root_db
            .database(MDB_DEV_MONGODB_DATABASE)
            .drop(None)
            .await?;

        // // Crear la base de datos nuevamente (esto creará una base de datos vacía)
        let create_command = doc! { "create": MDB_DEV_MONGODB_DATABASE };
        root_db
            .database(MDB_DEV_MONGODB_DATABASE)
            .run_command(create_command, None)
            .await?;
        let db = root_db.database(MDB_DEV_MONGODB_DATABASE);
        let collection = db.collection::<Activity>(MDB_DEV_MONGODB_COLLECTION);
        collection.drop(None).await?;
        db.create_collection(MDB_DEV_MONGODB_COLLECTION, None)
            .await?;

        info!("{:<12} - init_dev_mongodb()", "FOR-DEV-ONLY");
    } else {
        eprintln!("Error connecting to the MongoDB database.");
    }

    // let mm = ModelManager::new().await?;
    // let ctx = Ctx::root_ctx();

    Ok(())
}

pub async fn new_mongo_client(db_con_url: &str) -> Result<Client, mongodb::error::Error> {
    let mut client_options = ClientOptions::parse(db_con_url).await?;
    client_options.connect_timeout = Some(Duration::from_millis(500));
    let client = Client::with_options(client_options)?;
    Ok(client)
}

async fn pexec(db: &Db, file: &Path) -> Result<(), sqlx::Error> {
    info!("{:<12} - pexec: {file:?}", "FOR-DEV-ONLY");

    // -- Read the file.
    let content = fs::read_to_string(file)?;

    // FIXME: Make the split more sql proof.
    let sqls: Vec<&str> = content.split(';').collect();

    for sql in sqls {
        sqlx::query(sql).execute(db).await?;
    }

    Ok(())
}

async fn new_db_pool(db_con_url: &str) -> Result<Db, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_millis(500))
        .connect(db_con_url)
        .await
}
