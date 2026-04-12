use mongodb::bson::{self, doc};
use mongodb::results::DeleteResult;
use mongodb::results::InsertOneResult;
use rocket::State;
use rocket::futures::TryStreamExt;

use crate::models::Task;

pub async fn fetch_tasks(
    db: &mongodb::Database,
    user_id: &str,
    predicate: bson::Document,
) -> Vec<bson::Document> {
    let collection = db.collection::<bson::Document>(user_id);

    let cursor = collection.find(predicate).await.unwrap();

    cursor.try_collect().await.unwrap()
}

pub async fn insert_task(
    db: &mongodb::Database,
    user_id: &str,
    task: &Task,
) -> Result<InsertOneResult, mongodb::error::Error> {
    let collection = db.collection::<Task>(user_id);
    collection.insert_one(task).await
}

pub async fn delete_task(
    db: &mongodb::Database,
    user_id: &str,
    task_id: &str,
) -> Result<DeleteResult, mongodb::error::Error> {
    let collection = db.collection::<Task>(user_id);
    collection.delete_one(doc! { "id": task_id }).await
}
