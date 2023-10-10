use mongodb::{Client, options::ClientOptions};
use mongodb::bson::{doc, Document};
use log::{info, warn, error};
pub mod mongo_handler; 
use crate::mongo_handler::mongo_handler::{update_document, get_document};
use notify::{*};
use std::path::Path;

pub mod watch;
use crate::watch::watch::watch;


#[tokio::main]
async fn main() {
	env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let client_options = ClientOptions::parse("mongodb://fastquser:fastqpassword@localhost:27017/fastq").await.expect("Connection Error");
    let client = Client::with_options(client_options).unwrap();
    let db = "fastq" ;
    let collection = "run";

    let update_doc = doc! { "$set" : { "updated" : "true" } };

	let path = std::env::args()
			.nth(1)
			.expect("Argument 1 needs to be a path");

	log::info!("Watching {path}");

	if let Err(error) = watch(path) {
		log::error!("Error: {error:?}");
	}


    // let collection = db.collection::<Document>("run");
    // let doc = collection.find_one(filter, None).await.unwrap();
    // println!("{:?}", doc);

    // update_document(&client, db, collection, "test_updated", update_doc).await;
    // let entry = get_document(&client, db, collection, "test_updated").await;
    // info!("{}",entry.get("updated").unwrap());
}
