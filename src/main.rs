#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate chrono;
extern crate dotenv;
extern crate harsh;
extern crate rocket;
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;

use std::fs;
use std::fs::File;
use std::io;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::TimeZone;
use chrono::{DateTime, Utc};
use dotenv::dotenv;
use harsh::HarshBuilder;
use rocket::http::{RawStr, Status};
use rocket::response::Response;
use rocket::Data;
use rocket_contrib::Template;

#[derive(Serialize)]
struct TemplateContext {
    base_url: String,
    drops: Vec<Drop>,
}

#[derive(Serialize)]
struct Drop {
    id: String,
    created_on: DateTime<Utc>,
}

fn system_time_to_date_time(system_time: SystemTime) -> DateTime<Utc> {
    let (sec, nsec) = match system_time.duration_since(UNIX_EPOCH) {
        Ok(dur) => (dur.as_secs() as i64, dur.subsec_nanos()),
        Err(err) => {
            let dur = err.duration();
            let (sec, nsec) = (dur.as_secs() as i64, dur.subsec_nanos());
            if nsec == 0 {
                (-sec, 0)
            } else {
                (-sec - 1, 1_000_000_000 - nsec)
            }
        }
    };
    Utc.timestamp(sec, nsec)
}

fn list_drops() -> io::Result<Vec<Drop>> {
    let mut drops = Vec::new();
    for entry in fs::read_dir("upload")? {
        let entry = entry?;
        let id = entry.file_name().into_string().unwrap();
        let last_modified = system_time_to_date_time(entry.metadata()?.modified()?);
        drops.push(Drop {
            id,
            created_on: last_modified,
        });
    }
    Ok(drops)
}

#[get("/")]
fn index() -> io::Result<Template> {
    let drops = list_drops()?;

    let context = TemplateContext {
        base_url: String::from("http://localhost:8000"),
        drops,
    };

    Ok(Template::render("index", &context))
}

#[post("/", data = "<data>")]
fn upload(data: Data) -> io::Result<String> {
    let harsh = HarshBuilder::new().salt("滴").init().unwrap();
    let count = fs::read_dir("upload")?.count() as u64;
    let id = harsh.encode(&[count + 1]).unwrap();
    let filename = format!("upload/{id}", id = id);
    let url = format!("{host}/{id}\n", host = "http://localhost:8000", id = id);

    data.stream_to_file(Path::new(&filename))?;
    Ok(url)
}

#[get("/<id>")]
fn retrieve(id: &RawStr) -> Response {
    let filename = format!("upload/{id}", id = id);
    let file = File::open(&filename).unwrap();
    Response::build()
        .status(Status::Ok)
        .streamed_body(file)
        .finalize()
}

fn main() {
    dotenv().ok();

    rocket::ignite()
        .mount("/", routes![index, upload, retrieve])
        .attach(Template::fairing())
        .launch();
}
