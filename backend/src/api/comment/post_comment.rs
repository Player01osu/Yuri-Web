use crate::{
    api::comment::structs::ViewComments,
    database::mongo::{CollectionList, MongodbDatabase},
};
use common::mongodb::structs::{Comment, CommentSection};

use actix_web::{post, web, web::Data, web::Json, web::Path};
use bson::oid::ObjectId;
use mongodb::bson::doc;
use uuid::Uuid;

#[post("/post-comment/{post_id}")]
pub async fn post_comment(
    path: Path<ViewComments>,
    mut request: Json<Comment>,
    database: Data<MongodbDatabase>,
) -> actix_web::Result<Json<CommentSection>> {
    let comments_collection = database.collection::<CommentSection>(CollectionList::Comments);
    let query = doc! {
        "_id": ObjectId::parse_str(&path.post_id.as_str()).unwrap(),
    };

    if request.commenter.is_empty() {
        request.commenter = Uuid::new_v4().to_string();
    }

    let update = doc! {
        "$push": { "comments":
            {
                "commenter": &request.commenter,
                "body": &request.body,
            }
        }
    };

    let comment = comments_collection
        .find_one_and_update(query, update, None)
        .await
        .unwrap()
        .unwrap();

    Ok(web::Json(comment))
}
