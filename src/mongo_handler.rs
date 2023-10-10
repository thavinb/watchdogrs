
pub mod mongo_handler {
use mongodb::{Client, options::ClientOptions};
use mongodb::bson::{doc, Document};
use log::{info, warn, error};
    pub async fn get_document(client: &Client, db: &str, coll_name: &str, id: &str ) -> Document{
        let db = client.database(db);
        let collection = db.collection::<Document>(coll_name);
        let query = doc! { "name" : id };

        let result = collection.find_one(Some(query), None).await.unwrap();
        match result {
            Some(doc) => { doc },
            None => doc! {},
        }
    }

    pub async fn update_document(client: &Client, db: &str, coll_name: &str, id: &str, doc: Document ){
        let db = client.database(db);
        let collection = db.collection::<Document>(coll_name);
        let query = doc! { "name" : id };

        let update_result = collection.update_one(query, doc, None).await.unwrap();
        info!("{:?}",update_result);
        
}


}
