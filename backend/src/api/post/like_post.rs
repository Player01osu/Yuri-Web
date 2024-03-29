use common::mongodb::structs::YuriPosts;
use crate::database::mongo::{CollectionList, MongodbDatabase};

use actix_web::{put, web::{Data, Path}, HttpResponse};
use bson::oid::ObjectId;
use mongodb::bson::doc;

#[put("/like-post/{oid}")]
pub async fn like_post(
    path: Path<String>,
    database: Data<MongodbDatabase>,
) -> HttpResponse {
    // Parse oid into ObjectId object type
    let oid = ObjectId::parse_str(&path.as_str()).unwrap();
    let filter = doc! ("_id": oid);
    let add_like = doc! { "$inc": { "stats.likes": 1 } };
    let posts_collection = database.collection::<YuriPosts>(CollectionList::Posts);

    posts_collection
        .update_one(filter, add_like, None)
        .await
        .expect("Failed to add like");

    HttpResponse::Ok().body("HTTP/1.1 201 Updated")
}

#[put("/unlike-post")]
pub async fn unlike_post(
    path: Path<String>,
    database: Data<MongodbDatabase>,
) -> HttpResponse {
    // Parse oid into ObjectId object type
    let oid = ObjectId::parse_str(&path.as_str()).unwrap();
    let filter = doc! ("_id": oid);
    let remove_like = doc! { "$inc": { "stats.likes": -1 } };
    let posts_collection = database.collection::<YuriPosts>(CollectionList::Posts);

    posts_collection
        .update_one(filter, remove_like, None)
        .await
        .expect("Failed to remove like");

    HttpResponse::Ok().body("HTTP/1.1 201 Updated")
}
