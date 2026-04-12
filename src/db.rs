use mongodb::bson;
use rocket::State;
use rocket::futures::TryStreamExt;

pub async fn fetch_tasks(
    db: &State<mongodb::Database>,
    user_id: &str,
    predicate: bson::Document,
) -> Vec<bson::Document> {
    let collection = db.collection::<bson::Document>(user_id);

    let cursor = collection.find(predicate).await.unwrap();

    cursor.try_collect().await.unwrap()
}
