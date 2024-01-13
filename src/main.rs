use std::io::Write;
use mongodb::{Client, options::ClientOptions};
use mongodb::bson::{doc, Document};
use log::{Level, info, warn, error};
use chrono::Local;
use chrono::{TimeZone, Utc};
use env_logger::{Env, Builder, fmt::Color};
pub mod mongo_handler; 
use crate::mongo_handler::mongo_handler::{update_document, get_document};
use notify::{*};
use std::path::Path;
use std::thread; 

pub mod watch;
use crate::watch::watch::{watch, PathType};


#[tokio::main]
async fn main() {
	// logger setup 
	let env = Env::default()
        .filter_or("MY_LOG_LEVEL", "debug")
        .write_style_or("MY_LOG_STYLE", "always");

	let builder = Builder::from_env(env)
        .format(|buf, record| { 
				// Styling for level
				let mut level_style = buf.style();
				level_style.set_color(Color::Red).set_bold(true);
				match record.level() {
					Level::Trace => level_style.set_color(Color::White),
					Level::Debug => level_style.set_color(Color::Blue),
					Level::Info => level_style.set_color(Color::Green),
					Level::Warn => level_style.set_color(Color::Yellow),
					Level::Error => level_style.set_color(Color::Red),
				};
			
            writeln!(buf,
                "{} | {} | - {}",
                Local::now().format("%Y-%m-%dT%H:%M:%S"),
                level_style.value(record.level()),
                level_style.value(record.args())
            )
        })
		.write_style(env_logger::fmt::WriteStyle::Always)
        .init();


	// mongodb client setup
    let client_options = ClientOptions::parse("mongodb://fastquser:BkkG1%233689@localhost:27017/fastq")
        .await
        .expect("Fail Connection to mongoDB");
    let client = Client::with_options(client_options).unwrap();
    let db = "fastq" ;
    let collection = "run";
    let update_doc = doc! { "$set" : { "updated" : "true" } };

	// cli argument setup
	let path = std::env::args()
			.nth(1)
			.expect("Argument 1 needs to be a path");

	info!("Watching {path}");

	if let Err(error) = watch(path) {
		log::error!("Error: {error:?}");
	}

	// if let Err(err) = thread::spawn(|| watch(path)).join().unwrap() {
	// 		eprintln!("Error in notify watch loop: {:?}", err);
	// 	}
// use std::time::Duration;

	// loop {
	// 		thread::sleep(Duration::from_secs(1));
	// 	}


    // match watch(path) { 
    //     Ok(pathtype) => {
    //         if let PathType::File(e) = pathtype {
    //             log::info!("Yay");
    //         }
    //         else {
    //             // Dir file do nothing  
    //         }
    //     }
    //     Err(pathtype) => {
    //         // logerr 
    //     }
    // }


    // let collection = db.collection::<Document>("run");
    // let doc = collection.find_one(filter, None).await.unwrap();
    // println!("{:?}", doc);

    // update_document(&client, db, collection, "test_updated", update_doc).await;
    // let entry = get_document(&client, db, collection, "test_updated").await;
    // info!("{}",entry.get("updated").unwrap());
}
