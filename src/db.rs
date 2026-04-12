use mongodb::{
    Database,
    bson::{self, doc},
    results::{DeleteResult, InsertOneResult},
};
use rocket::futures::TryStreamExt;

use crate::models::Task;

pub async fn fetch_tasks(
    db: &Database,
    user_id: &str,
    predicate: bson::Document,
) -> Vec<bson::Document> {
    let collection = db.collection::<bson::Document>(user_id);

    let cursor = collection.find(predicate).await.unwrap();

    cursor.try_collect().await.unwrap()
}

pub async fn insert_task(
    db: &Database,
    user_id: &str,
    task: &Task,
) -> Result<InsertOneResult, mongodb::error::Error> {
    let collection = db.collection::<Task>(user_id);
    collection.insert_one(task).await
}

pub async fn delete_task(
    db: &Database,
    user_id: &str,
    task_id: &str,
) -> Result<DeleteResult, mongodb::error::Error> {
    let collection = db.collection::<Task>(user_id);
    collection.delete_one(doc! { "id": task_id }).await
}
