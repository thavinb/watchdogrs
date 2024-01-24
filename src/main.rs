use std::io::Write;
use mongodb::Client;
use mongodb::bson::doc;
use log::Level;
use chrono::Local;
use env_logger::{Env, Builder, fmt::Color};
pub mod mongo; 
use crate::mongo::mongo_handler::{update_document, insert_document, connect_db, delete_document};
use notify::{*};
use std::path::Path;
use std::thread; 
use std::time::Duration;
use futures::executor::block_on;

pub mod watch;


use watch::utils_checkfile::{PathType, FileType, handle_create_event, handle_modify_event, handle_remove_event};


fn main() {
	// logger setup 
	let env = Env::default()
        .filter_or("MY_LOG_LEVEL", "debug")
        .write_style_or("MY_LOG_STYLE", "always");

	let _builder = Builder::from_env(env)
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
    let client_options =  block_on(
		connect_db(
			"mongodb://fastquser:BkkG1%233689@localhost:27017/dnall"
		));
	// ClientOptions::parse("mongodb://fastquser:BkkG1%233689@localhost:27017/dnall")
    //     .await
	// 	.expect("Fail Connection to mongoDB");
    let client = Client::with_options(client_options).unwrap();
    let db = "dnall" ;
    let collection = "file";
    // let update_doc = doc! { "$set" : { "updated" : "false" } };
	
	
	
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
			let event = poll_event.expect("Fatal: watcher err at unwrapping the event");
			let watch_path = Path::new(&watched_path);
				match event.kind {
					EventKind::Create(_kind) => {

						let event_handler = handle_create_event(watch_path.as_ref(), event);
						if let PathType::File(path) = event_handler {
							match path {
								Ok(fileinfo) => {
									log::debug!("Calling POST mongo");
									let c = client.clone();
									
									
									block_on(insert_document(&c, db, collection, fileinfo));
									
        
									
									
								}
								Err(fileerror) => {
									log::warn!("{:?}",fileerror);
								}
							}
						}

							
						// TODO: handle struct then upload  
						// TODO: handle mongo status

					}
					
					EventKind::Modify(_modify_kind) => {
					    // If path is dir
					    // Check complete/incomplete tag
					    //     Warn if incomplete!
					    // else if file
					    // check if it is incomplete fastq file 
					    // if not warn!
						let event_handler = handle_modify_event(watch_path.as_ref(), event);
						if let PathType::File(path) = event_handler {
							match path {
								Ok(fileinfo) => {
									if let FileType::FQ(mate_number, _file_status) = fileinfo.file_type {
										
										// Use file path as key
										

										let query = doc! { 
											"basename"  : fileinfo.basename, 
											"file_path" : fileinfo.file_path,
											"file_type" : { 
												"FQ" : [ mate_number, Some("Pending") ] 
											},
											"status" : Some("Pending")
										};

										let update_doc = doc! {
											"$set" : {
												"file_size" : fileinfo.file_size as i64
											}
										};

										log::debug!("Calling POST mongo");
										log::debug!("Update entry with query {:?}", &query);
										let c = client.clone();
										block_on(update_document(&c, db, collection, query, update_doc));
										
									} else { 
										log::warn!("unexpected file is modified: {:?}", fileinfo.file_type );
									}
								}
								Err(fileerror) => {
									log::warn!("{:?}",fileerror);
								}
							}
						} else {
							()
						}

						
					}
					EventKind::Remove(_remove_kind) => {
					    // If path is dir
					    // Warn!
					    // else if file 
					    // check if it is incomplete fastq file
					    // if not warn!
						let event_handler = handle_remove_event(watch_path.as_ref(), event);
						if let PathType::File(path) = event_handler {
							match path {
								Ok(fileinfo) => {
									if let FileType::FQ(mate_number, _file_status) = fileinfo.file_type {
										// Either rsync fail or finish, 
										// The temp file is remove
										// TODO: get the finish document first
										// TODO: IF  return None then remove file
										// TODO: Else label it as removed

										let query = doc! { 
											"basename"  : fileinfo.basename, 
											"file_type" : { 
												"FQ" : [ mate_number, Some("Pending") ] 
											},
											"status" : Some("Pending")
											
										};
										log::debug!("Calling DELETE mongo");
										log::debug!("Delete entry with query {:?}", &query);
										let c = client.clone();
										block_on(delete_document(&c, db, collection, query));
										
									} else { 
										log::warn!("unexpected file is removed: {:?}", fileinfo.file_type );
										let query = doc! { 
											"basename"  : fileinfo.basename, 
											"file_type" : fileinfo.file_path											
										};
										let update_doc = doc! {
											"$set" : {
												"status" : "Removed"
											}
										};
										let c = client.clone();
										block_on(update_document(&c, db, collection, query, update_doc));
										
									}									
								}
								Err(fileerror) => {
									log::warn!("{:?}",fileerror);
								}
							}
						}
					},
					// EventKind::Access(AccessKind) => {}
					// EventKind::Any => {}
					// EventKind::Other => {}
					// // let data = check_event(&path, event).unwrap();
					// // data update
					// // mongohandiling(package, event)
					_ => {
						drop(PathType::None);
					}
			}
		}
	);
			// data checkfile
			// mongohandling(package, scan)

	loop{}
    // running notify server
	



}
