use actix_multipart::Multipart;
use actix_web::http::StatusCode;
use actix_web::{delete, get, post, put, web::Data, web::Json, web::Path};
use actix_web::{web, Error, HttpResponse, HttpResponseBuilder, Responder};
use bson::oid::ObjectId;
use futures_util::TryStreamExt as _;
use image::io::Reader;
use mongodb::bson::Document;
use mongodb::{bson::doc, options::FindOptions};
use serde::{Deserialize, Serialize};
use std::io::Write;
use uuid::Uuid;

use super::mongo::MongodbDatabase;
use common::mongodb::structs::{Comment, PostStats, Resolution, Source, YuriPosts};

#[derive(Deserialize, Serialize)]
pub struct TaskIndentifier {
    task_global_id: String,
}

pub struct Gallery {
    show: Option<Json<Vec<Document>>>,
    search_filters: Option<Vec<String>>,
    amount: u16,
}

impl Gallery {
    pub fn new(amount: u16) -> Gallery {
        let generated = Gallery {
            show: None,
            search_filters: None,
            amount,
        };
        generated
    }

    async fn gen_gallery(
        &mut self,
        database: Data<mongodb::Collection<YuriPosts>>,
        sort: String,
    ) -> &mut Self {
        let database = MongodbDatabase::new(database);

        let find_options = match sort.as_str() {
            "new" => FindOptions::builder()
                .limit(i64::from(self.amount))
                .sort(doc! {"time":-1})
                .build(),
            "top" => FindOptions::builder()
                .limit(i64::from(self.amount))
                .sort(doc! {"stats.likes":-1, "time":-1})
                .build(),
            "views" => FindOptions::builder()
                .limit(i64::from(self.amount))
                .sort(doc! {"stats.views":-1, "time":-1})
                .build(),
            _ => FindOptions::builder()
                .limit(i64::from(self.amount))
                .sort(doc! {"time":-1})
                .build(),
        };

        let paths: Vec<Document> = database.find(None, Some(find_options), self.amount).await;

        match paths.is_empty() {
            true => {
                self.show = None;
                self
            }
            false => {
                self.show = Some(Json(paths));
                self
            }
        }
    }
}

#[get("/view-posts/{page_number}/{sort}")]
pub async fn view_posts(
    path: Path<(u16, String)>,
    database: Data<mongodb::Collection<YuriPosts>>,
) -> Json<Vec<Document>> {
    let (page_number, sort) = path.into_inner();

    let mut generated = Gallery::new(page_number * 10);

    generated.gen_gallery(database, sort).await;

    match generated.show {
        Some(documents) => documents,
        None => Json(Vec::default())
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PostImageRequest {
    title: String,
    author: String,
    op: String,
    source: Source,
    resolution: Resolution,
    time: u64,
    tags: Option<Vec<String>>,
    file_name: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DeleteImageRequest {
    oid: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct LikeImageRequest {
    oid: String,
}

#[post("/post_image")]
pub async fn post_image(
    database: Data<mongodb::Collection<YuriPosts>>,
    mut payload: Multipart,
) -> actix_web::Result<impl Responder> {
    let database: mongodb::Collection<YuriPosts> = database.clone_with_type();
    let utc_now = chrono::Utc::now();
    let time = utc_now.timestamp() as u64;

    // FIXME: This is probably not good
    let mut title = String::new();
    let mut author = String::new();
    let mut op = String::default();
    let mut material = String::new();
    let mut link: Option<String> = None;
    let mut width: usize = 480;
    let mut height: usize = 640;
    let mut tags: Option<Vec<String>> = None;
    let mut filename = String::default();
    let mut path = String::default();

    while let Some(mut field) = payload.try_next().await? {
        match field.name() {
            "title" => {
                if let Some(chunk) = field.try_next().await? {
                    title = std::str::from_utf8(&chunk)?.to_owned();
                }
            }
            "author" => {
                if let Some(chunk) = field.try_next().await? {
                    author = std::str::from_utf8(&chunk)?.to_owned();
                }
            }
            "op" => {
                if let Some(chunk) = field.try_next().await? {
                    let chunk_to_str = std::str::from_utf8(&chunk)?; // FIXME: Doesn't default
                                                                     // empty
                    op = match chunk_to_str.is_empty() {
                        true => String::from("an"),
                        false => String::from(chunk_to_str),
                    };
                }
            }
            "material" => {
                if let Some(chunk) = field.try_next().await? {
                    material = std::str::from_utf8(&chunk)?.to_owned();
                }
            }
            "link" => {
                if let Some(chunk) = field.try_next().await? {
                    let chunk_to_str = std::str::from_utf8(&chunk)?;
                    link = match chunk_to_str.is_empty() {
                        true => None,
                        false => Some(String::from(chunk_to_str)),
                    };
                }
            }
            "tags" => {
                if let Some(chunk) = field.try_next().await? {
                    let chunk_to_str = std::str::from_utf8(&chunk)?;
                    tags = match chunk_to_str.is_empty() {
                        true => None,
                        false => Some(
                            String::from(chunk_to_str)
                                .split_terminator(',')
                                .map(|s| String::from(s.trim()))
                                .collect::<Vec<String>>(),
                        ),
                    };
                }
            }
            "filename" => {
                if let Some(chunk) = field.try_next().await? {
                    filename = std::str::from_utf8(&chunk)?.to_owned();
                    filename = sanitize_filename::sanitize(filename);
                }
            }
            "image" => {
                // A multipart/form-data stream has to contain `content_disposition`
                let filepath = format!("./assets/posts/{author}-{time}-{filename}");
                path = filepath.clone();
                let mut raw_data: Vec<u8> = Vec::default();

                let mut f = web::block(|| std::fs::File::create(filepath)).await??;

                while let Some(chunk) = field.try_next().await? {
                    raw_data.extend(chunk.iter().clone()); // FIXME: This is bad... I think?
                    f = web::block(move || f.write_all(&chunk).map(|_| f)).await??;
                }
                let dimensions = web::block(move || {
                    Reader::new(std::io::Cursor::new(raw_data))
                        .with_guessed_format()
                        .unwrap()
                        .into_dimensions()
                })
                .await?
                .unwrap();
                width = dimensions.0 as usize;
                height = dimensions.1 as usize;
            }
            _ => (),
        }
    }

    let resolution = Resolution { width, height };

    let stats = PostStats { likes: 0, views: 0 };

    let source = Source { material, link };
    //
    // TODO: Reference counted?
    let docs = YuriPosts {
        title,
        author,
        op,
        path,
        source,
        resolution,
        time,
        tags,
        stats,
        comments: None,
    };

    let oid = database
        .insert_one(docs, None)
        .await
        .expect("Handle this error properly u lazy fuck")
        .inserted_id
        .as_object_id()
        .expect("Could not convert to ObjectId")
        .to_hex();

    Ok(web::Json(DeleteImageRequest { oid }))
}

#[delete("/delete_post")]
pub async fn delete_post(
    database: Data<mongodb::Collection<YuriPosts>>,
    request: Json<DeleteImageRequest>,
) -> HttpResponse {
    let oid = ObjectId::parse_str(&request.oid.as_str()).unwrap();
    let filter = doc! {
        "_id": oid,
    };

    let post_struct = database
        .find_one(filter.clone(), None)
        .await
        .expect("BRO WHAT DA HECK")
        .expect("Unable to find post from ObjectId");

    std::fs::remove_file(post_struct.path).unwrap_or(println!("Unable to remove file"));

    database.delete_one(filter, None).await.expect("SHITTTT");

    HttpResponse::Ok().body("Deleted")
}

#[put("/like-post")]
pub async fn like_post(
    request: Json<LikeImageRequest>,
    database: Data<mongodb::Collection<YuriPosts>>,
) -> HttpResponse {
    // Parse oid into ObjectId object type
    let oid = ObjectId::parse_str(&request.oid.as_str()).unwrap();
    let filter = doc! {
        "_id": oid,
    };
    let add_like = doc! { "$inc": { "stats.likes": 1 } };

    database
        .update_one(filter, add_like, None)
        .await
        .expect("Failed to add like");

    HttpResponse::Ok().body("HTTP/1.1 201 Updated")
}

#[put("/unlike-post")]
pub async fn unlike_post(
    request: Json<LikeImageRequest>,
    database: Data<mongodb::Collection<YuriPosts>>,
) -> HttpResponse {
    // Parse oid into ObjectId object type
    let oid = ObjectId::parse_str(&request.oid.as_str()).unwrap();
    let filter = doc! {
        "_id": oid,
    };
    let remove_like = doc! { "$inc": { "stats.likes": -1 } };

    database
        .update_one(filter, remove_like, None)
        .await
        .expect("Failed to remove like");

    HttpResponse::Ok().body("HTTP/1.1 201 Updated")
}
