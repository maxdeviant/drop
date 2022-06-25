use rocket_db_pools::{sqlx, Database};

#[derive(Database)]
#[database("drop")]
pub struct Db(sqlx::SqlitePool);

pub type Result<T, E = rocket::response::Debug<sqlx::Error>> = std::result::Result<T, E>;
