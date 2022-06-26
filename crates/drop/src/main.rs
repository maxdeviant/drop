#[macro_use]
extern crate rocket;

mod database;
mod domain;

use std::env;
use std::path::Path;

use aws_sdk_s3::types::ByteStream;
use aws_sdk_s3::Client;
use chrono::{DateTime, Datelike, SecondsFormat, Utc};
use dotenv::dotenv;
use rocket::data::ToByteUnit;
use rocket::http::ContentType;
use rocket::outcome::{try_outcome, Outcome};
use rocket::request::FromRequest;
use rocket::request::{self, FromParam};
use rocket::serde::json::Json;
use rocket::serde::Serialize;
use rocket::{Data, Request};
use rocket_db_pools::{Connection, Database};
use tokio::fs;

use database::Db;
use domain::entities::{ApiKeyId, DropId, User, UserId};
use domain::ApiKeyValue;

impl<'a> FromParam<'a> for DropId {
    type Error = String;

    fn from_param(param: &'a str) -> Result<Self, Self::Error> {
        DropId::try_from(param)
            .map_err(|err| format!("failed to parse '{}' as a drop ID: {}", param, err))
    }
}

impl<'a> FromParam<'a> for UserId {
    type Error = String;

    fn from_param(param: &'a str) -> Result<Self, Self::Error> {
        UserId::try_from(param)
            .map_err(|err| format!("failed to parse '{}' as a user ID: {}", param, err))
    }
}

pub struct ApiKeyBearer {
    user: User,
}

struct ApiKeyLookup {
    pub id: String,
    pub username: String,
    pub full_name: Option<String>,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApiKeyBearer {
    type Error = String;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let mut db = try_outcome!(req
            .guard::<Connection<Db>>()
            .await
            .map_failure(|f| (f.0, "failed to acquire database connection".to_string())));

        match req.headers().get_one("Authorization") {
            Some(authorization) => {
                let token = authorization.replace("Bearer ", "");

                let api_key = sqlx::query_as!(
                    ApiKeyLookup,
                    "
                    select user.id, user.username, user.full_name from api_key
                    inner join user on user.id = api_key.user_id
                    where api_key.value = $1
                    ",
                    token
                )
                .fetch_one(&mut *db)
                .await
                .unwrap();

                Outcome::Success(ApiKeyBearer {
                    user: User {
                        id: UserId::try_from(api_key.id).unwrap(),
                        username: api_key.username,
                        full_name: api_key.full_name,
                    },
                })
            }
            _ => Outcome::Forward(()),
        }
    }
}

#[get("/")]
async fn index() -> &'static str {
    "滴"
}

#[post("/drops", data = "<drop>")]
async fn upload_drop(_bearer: ApiKeyBearer, drop: Data<'_>) -> std::io::Result<String> {
    let id = DropId::new();

    let temp_dir = Path::new("temp");
    fs::create_dir_all(temp_dir).await.unwrap();

    let temp_filepath = temp_dir.join(id.to_string());

    drop.open(2.mebibytes()).into_file(&temp_filepath).await?;

    let s3_config = aws_config::load_from_env().await;
    let client = Client::new(&s3_config);

    let body = ByteStream::from_path(&temp_filepath).await?;

    client
        .put_object()
        .bucket(env::var("AWS_S3_BUCKET").unwrap())
        .key(format!(
            "drops/{}/{}",
            id.created_at().date().year(),
            id.unprefixed()
        ))
        .body(body)
        .send()
        .await
        .expect("failed to upload to S3");

    tokio::fs::remove_file(temp_filepath).await.ok();

    Ok(id.to_string())
}

#[get("/drops/<id>")]
async fn get_drop(id: DropId) -> (ContentType, Vec<u8>) {
    let s3_config = aws_config::load_from_env().await;
    let client = Client::new(&s3_config);

    let object = client
        .get_object()
        .bucket(env::var("AWS_S3_BUCKET").unwrap())
        .key(format!(
            "drops/{}/{}",
            id.created_at().date().year(),
            id.unprefixed()
        ))
        .send()
        .await
        .expect("failed to retrieve object from S3");

    let bytes = object
        .body
        .collect()
        .await
        .map(|data| data.into_bytes())
        .unwrap()
        .to_vec();

    let mime_type = tree_magic_mini::from_u8(&bytes);

    let content_type = ContentType::parse_flexible(mime_type).unwrap_or(ContentType::Binary);

    (content_type, bytes)
}

#[derive(Debug, Serialize)]
pub struct DropMetadata {
    pub created_at: DateTime<Utc>,
    pub size_in_bytes: i64,
}

#[get("/drops/<id>/meta")]
async fn get_drop_metadata(id: DropId) -> Json<DropMetadata> {
    let s3_config = aws_config::load_from_env().await;
    let client = Client::new(&s3_config);

    let object = client
        .get_object()
        .bucket(env::var("AWS_S3_BUCKET").unwrap())
        .key(format!(
            "drops/{}/{}",
            id.created_at().date().year(),
            id.unprefixed()
        ))
        .send()
        .await
        .expect("failed to retrieve object from S3");

    Json(DropMetadata {
        created_at: id.created_at(),
        size_in_bytes: object.content_length,
    })
}

#[derive(Debug, Serialize)]
pub struct UserJson {
    pub id: String,
    pub username: String,
    pub full_name: Option<String>,
}

#[post("/users")]
async fn create_user(
    _bearer: ApiKeyBearer,
    mut db: Connection<Db>,
) -> database::Result<Json<UserJson>> {
    let id = UserId::new();

    let username = "maxdeviant";
    let full_name = "Marshall Bowers";

    let unprefixed_id = id.unprefixed();
    let now = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);

    sqlx::query!(
        "
        insert into user (id, created_at, updated_at, username, full_name)
        values (?, ?, ?, ?, ?)
        ",
        unprefixed_id,
        now,
        now,
        username,
        full_name
    )
    .execute(&mut *db)
    .await?;

    Ok(Json(UserJson {
        id: id.to_string(),
        username: username.to_string(),
        full_name: Some(full_name.to_string()),
    }))
}

#[derive(Debug, Serialize)]
pub struct ApiKeyJson {
    pub id: String,
    pub value: String,
}

#[post("/keys")]
async fn generate_api_key(
    bearer: ApiKeyBearer,
    mut db: Connection<Db>,
) -> database::Result<Json<ApiKeyJson>> {
    let id = ApiKeyId::new();
    let value = ApiKeyValue::new();

    let unprefixed_id = id.unprefixed();
    let unprefixed_user_id = bearer.user.id.unprefixed();
    let key_value = value.to_string();
    let now = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);

    sqlx::query!(
        "
        insert into api_key (id, created_at, updated_at, user_id, value)
        values (?, ?, ?, ?, ?)
        ",
        unprefixed_id,
        now,
        now,
        unprefixed_user_id,
        key_value
    )
    .execute(&mut *db)
    .await?;

    Ok(Json(ApiKeyJson {
        id: id.to_string(),
        value: value.to_string(),
    }))
}

async fn ensure_root_user_exists() {
    use sqlx::{Connection, SqliteConnection};

    let root_username = env::var("ROOT_USERNAME").unwrap();
    let root_api_key = env::var("ROOT_API_KEY").unwrap();

    let database_url = env::var("DATABASE_URL").unwrap();
    let mut connection = SqliteConnection::connect(&database_url).await.unwrap();

    let user_id = UserId::new();
    let unprefixed_user_id = user_id.unprefixed();
    let now = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);

    #[derive(Debug)]
    struct RootUser {
        id: String,
    }

    let root_user = sqlx::query_as!(
        RootUser,
        "
        insert into user (id, created_at, updated_at, username)
        values (?, ?, ?, ?)
        -- This is a no-op update just so we can get the `id` returned.
        on conflict (username) do update set updated_at = updated_at
        returning id
        ",
        unprefixed_user_id,
        now,
        now,
        root_username
    )
    .fetch_one(&mut connection)
    .await
    .expect("failed to insert root user");

    println!("Root user: {:?}", root_user);

    let key_id = ApiKeyId::new();
    let unprefixed_key_id = key_id.unprefixed();
    let now = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);

    sqlx::query!(
        "
        insert into api_key (id, created_at, updated_at, user_id, value)
        values (?, ?, ?, ?, ?)
        on conflict (value) do nothing
        ",
        unprefixed_key_id,
        now,
        now,
        root_user.id,
        root_api_key
    )
    .execute(&mut connection)
    .await
    .unwrap();
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    dotenv().ok();

    ensure_root_user_exists().await;

    let rocket = rocket::build()
        .mount(
            "/",
            routes![index, upload_drop, get_drop, get_drop_metadata,],
        )
        .mount("/x/", routes![create_user, generate_api_key])
        .attach(Db::init())
        .ignite()
        .await?;

    let _ = rocket.launch().await?;

    Ok(())
}
