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
use std::time::Duration;

pub mod watch;

use watch::utils_checkfile::{PathType, handle_create_event, handle_modify_event, handle_remove_event};


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
	let watched_path = std::env::args()
			.nth(1)
			.expect("Argument 1 needs to be a path");

	
	// Config watcher
	let config = Config::default()
		.with_poll_interval(Duration::from_secs(10));
		// .with_compare_contents(true);
	let (tx, rx) = std::sync::mpsc::channel();

	let mut watcher = PollWatcher::with_initial_scan(
		tx,
		config,
		move |scan_event| {
			log::info!("Scan event {scan_event:?}");
		},
	).unwrap();

	log::info!("Watching {watched_path}");
	if let Err(error) = watcher.watch(watched_path.as_ref(), RecursiveMode::Recursive) {
		log::error!("Error: {error:?}");
	}
	thread::spawn( move ||
		for poll_event in rx {
			let mut event = poll_event.expect("Fatal: watcher err at unwrapping the event");
			let watch_path = Path::new(&watched_path);
				match event.kind {
					EventKind::Create(kind) => {
						handle_create_event(watch_path.as_ref(), event);
							
						// TODO: handle struct then upload  
						// TODO: handle mongo status

					}
					
					EventKind::Modify(ModifyKind) => {
					    // If path is dir
					    // Check complete/incomplete tag
					    //     Warn if incomplete!
					    // else if file
					    // check if it is incomplete fastq file 
					    // if not warn!
						handle_modify_event(watch_path.as_ref(), event);
					}
					EventKind::Remove(RemoveKind) => {
					    // If path is dir
					    // Warn!
					    // else if file 
					    // check if it is incomplete fastq file
					    // if not warn!
						handle_remove_event(watch_path.as_ref(), event);
					},
					// EventKind::Access(AccessKind) => {}
					// EventKind::Any => {}
					// EventKind::Other => {}
					// // let data = check_event(&path, event).unwrap();
					// // data update
					// // mongohandiling(package, event)
					_ => {
						PathType::None;
					}
			}
		}
	);
			// data checkfile
			// mongohandling(package, scan)

	loop{}
    // running notify server
	



}
