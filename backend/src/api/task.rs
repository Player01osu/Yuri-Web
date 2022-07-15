use actix_web::http::StatusCode;
use actix_web::web::JsonBody;
use actix_web::{delete, get, post, web::Data, web::Json, web::Path};
use actix_web::{HttpResponse, HttpResponseBuilder};

use serde::{Deserialize, Serialize};

use mongodb::bson::Document;

use super::mongo::{self, MongodbCollection, MongodbDatabase};

#[derive(Deserialize, Serialize)]
pub struct TaskIndentifier {
    task_global_id: String,
}

use common::mongodb::structs::YuriPosts;
use mongodb::{bson::doc, options::FindOptions};

pub struct Gallery {
    show: Option<Json<Vec<Document>>>,
    search_filters: Option<String>,
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

    async fn gen_gallery(&mut self, database: Data<mongodb::Collection<YuriPosts>>) -> &mut Self {
        // >query mongodb for 'yuriPosts'
        // >find a mix of new posts and
        // most viewed posts
        // >limit to around 20 posts
        // >return json of them

        let database = MongodbDatabase::new(database);
        let filter = doc! { "author": "Player01" };
        let find_options = FindOptions::builder()
            .sort(doc! { "_id": i32::from(1) })
            .build();

        let paths: Vec<Document> = database.find(filter, Some(find_options), self.amount).await;

        if paths.is_empty() {
            self.show = None;
            return self;
        }

        self.show = Some(Json(paths));

        self
    }
}

#[get("/gallery_display")]
pub async fn gallery_display(
    database: Data<mongodb::Collection<YuriPosts>>,
) -> Json<Vec<Document>> {
    let mut generated = Gallery::new(20);
    generated.gen_gallery(database).await;

    return generated.show.unwrap();
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PostImageRequest {
    title: String,
    author: String,
    op: String,
    time: u64,
    tags: Vec<String>,
    file_name: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DeleteImageRequest {
    title: String,
}

#[post("/post_image")]
pub async fn post_image(
    database: Data<mongodb::Collection<YuriPosts>>,
    request: Json<PostImageRequest>,
) -> HttpResponse {
    let path = format!(
        "./assets/posts/{}-{}-{}",
        &request.author, &request.time, &request.file_name
    );
    let database: mongodb::Collection<YuriPosts> = database.clone_with_type();

    let docs = YuriPosts {
        title: request.title.clone(),
        op: request.op.clone(),
        path,
        time: request.time.clone(),
        author: request.author.clone(),
        tags: request.tags.clone(),
    };

    database
        .insert_one(docs, None)
        .await
        .expect("Handle this error properly u lazy fuck");

    HttpResponse::Ok().body("yeet")
}

#[delete("/delete_post")]
pub async fn delete_post(
    database: Data<mongodb::Collection<YuriPosts>>,
    request: Json<DeleteImageRequest>,
) -> HttpResponse {
    let filter = doc! { "title": format!("{}", &request.title) };

    database
        .delete_one(filter, None)
        .await
        .expect("Handle this pweeze");

    HttpResponse::Ok().body("Deleted")
}
