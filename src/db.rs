use mongodb::options::FindOneAndUpdateOptions;
use mongodb::{
    Database,
    bson::{self, doc},
    error::Error,
    options::ReturnDocument,
    results::{DeleteResult, InsertOneResult},
};
use rocket::futures::TryStreamExt;

use crate::models::Task;

fn coll() -> &'static str {
    "user_tasks"
}

pub async fn fetch_tasks(db: &Database, predicate: bson::Document) -> Vec<Task> {
    let collection = db.collection::<Task>(coll());

    let cursor = collection.find(predicate).await.unwrap();

    cursor.try_collect().await.unwrap()
}

pub async fn fetch_task(db: &Database, predicate: bson::Document) -> Result<Option<Task>, Error> {
    let collection = db.collection::<Task>(coll());

    collection.find_one(predicate).await
}

pub async fn insert_task(db: &Database, task: &Task) -> Result<InsertOneResult, Error> {
    let collection = db.collection::<Task>(coll());
    collection.insert_one(task).await
}

pub async fn delete_task(
    db: &Database,
    user_id: &str,
    task_id: &str,
) -> Result<DeleteResult, Error> {
    let collection = db.collection::<Task>(coll());
    collection
        .delete_one(doc! { "id": task_id, "user_id": user_id })
        .await
}

pub async fn modify_task(
    db: &Database,
    user_id: &str,
    task_id: &str,
    parameter: &str,
    new_state: impl Into<bson::Bson>,
) -> Result<Option<Task>, Error> {
    let collection = db.collection::<Task>(coll());
    let options = FindOneAndUpdateOptions::builder()
        .return_document(ReturnDocument::After)
        .build();
    collection
        .find_one_and_update(
            doc! {"id": task_id, "user_id": user_id },
            doc! { "$set": { parameter: new_state } },
        )
        .with_options(options)
        .await
}

pub async fn get_unique_projects(db: &Database, user_id: &str) -> Result<Vec<bson::Bson>, Error> {
    let collection = db.collection::<Task>(coll());

    collection
        .distinct("project", doc! { "user_id": user_id })
        .await
}

pub async fn toggle_completed_state(
    db: &Database,
    user_id: &str,
    task_id: &str,
) -> Result<Option<Task>, Error> {
    let collection = db.collection::<Task>(coll());
    let options = FindOneAndUpdateOptions::builder()
        .return_document(ReturnDocument::After)
        .build();
    collection
        .find_one_and_update(
            doc! {"id": task_id, "user_id": user_id},
            vec![doc! {
                "$set": {
                    "completed": { "$not": ["$completed"]}
                }
            }],
        )
        .with_options(options)
        .await
}
