#[macro_use]
extern crate rocket;

#[get("/")]
async fn index() -> &'static str {
    "滴"
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let rocket = rocket::build().mount("/", routes![index]).ignite().await?;

    let _ = rocket.launch().await?;

    Ok(())
}
