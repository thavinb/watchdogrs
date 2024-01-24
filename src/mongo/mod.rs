pub mod mongo_handler {
    use crate::watch::utils_checkfile::{FileInfo};
    use mongodb::{Client, options::ClientOptions, Collection, Database, };
    use mongodb::bson::{doc, Document};
    use log::{info, warn, error};

    pub async fn connect_db(url: &str) -> ClientOptions {
        ClientOptions::parse(url)
            .await
            .expect("Fail to connect to mongodb server.")
    } 
    pub async fn get_document(client: &Client, db: &str, coll_name: &str, id: &str ) -> Document {
        let db = client.database(db);
        let collection = db.collection::<Document>(coll_name);
        let query = doc! { "name" : id };

        let result = collection.find_one(Some(query), None).await.unwrap();
        match result {
            Some(doc) => { doc },
            None => doc! {},
        }
    }

    pub async fn update_document(client: &Client, db: &str, coll_name: &str, query: Document, doc: Document){
        let db = client.database(db);
        let collection: Collection<FileInfo> = db.collection(coll_name);
        // let query = doc! { "name" : id };
        log::info!("PUT entry in mongodb");
        let update_result = collection.update_one(query, doc, None).await.unwrap();
        info!("{:?}",update_result);
        
    }
   
    pub async fn insert_document(client: &Client, db: &str, coll_name: &str,  doc: FileInfo ){
        let db = client.database(db);
        // let collection = db.collection::<Document>(coll_name);
        let collection: Collection<FileInfo> = db.collection(coll_name);
        log::info!("POST entry to mongodb");
        
        let update_result = collection.insert_one(doc, None).await;
        info!("{:?}",update_result.unwrap());
    }

    pub async fn delete_document(client: &Client, db: &str, coll_name: &str,  query: Document ){
        let db = client.database(db);
        // let collection = db.collection::<Document>(coll_name);
        let collection: Collection<FileInfo> = db.collection(coll_name);
        log::info!("DELETE entry in mongodb");
        
        let update_result = collection.delete_one(query, None).await;
        info!("{:?}",update_result.unwrap());
    }

        
}




// pub mod data_transformation {
//     use serde::{Serialize, Deserialize};
//     use bson::{bson, Document};
//     let bson_doc: Document = bson::to_document(&my_instance).expect("Failed to serialize to BSON");
            
    


// }
