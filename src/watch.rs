pub mod watch {
	use notify::{poll::ScanEvent, Config, PollWatcher, RecursiveMode, Watcher, event};
	use std::path::Path;
	use std::time::Duration;
	use std::{fs, io};
    use std::error::Error;
    use log::{info, warn, error};
    use std::path::PathBuf ;
    use std::collections::HashMap;
    use notify::event::EventKind;
    use regex::{Regex, RegexSet};
    use std::os::unix::fs::MetadataExt;
    use md5::{Md5, Digest};
    use hex_literal::hex;

	pub fn watch<P: AsRef<Path>>(path: P) -> notify::Result<()> {
		let config = Config::default()
			.with_poll_interval(Duration::from_secs(10));
		// .with_compare_contents(true);
		let (tx, rx) = std::sync::mpsc::channel();

		// if you want to use the same channel for both events
		// and you need to differentiate between scan and file change events,
		// then you will have to use something like this
		enum Message {
			Event(notify::Result<notify::Event>),
			Scan(ScanEvent),
		}

		let tx_c = tx.clone();
		// use the pollwatcher and set a callback for the scanning events
		let mut watcher = PollWatcher::with_initial_scan(
			move |watch_event| {
				tx_c.send(Message::Event(watch_event)).unwrap();
			},
			// Config::default().with_poll_interval(Duration::from_secs(2)),
			config,
			move |scan_event| {
				tx.send(Message::Scan(scan_event)).unwrap();
			},
		)?;

		// Add a path to be watched. All files and directories at that path and
		// below will be monitored for changes.
		watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;
		for res in &rx {
			match res {
				Message::Event(e) => {
                    match e {
                        Ok(event) => {
                            match event.kind {
                                EventKind::Create(kind) => {
                                    handle_create_event(&path.as_ref(), event);
                                    // match handle_create_event(&path.as_ref(), event) {
                                    //     PathType::Dir(result) => {
                                    //         if let Err(e) = result {
                                    //             log::warn!("{:?}",e);
                                    //         } else {
                                    //             ()
                                    //         }
                                    //     }
                                    //     PathType::File(x) => {
                                    //        // TODO: handle struct then upload  
                                    //        // TODO: handle mongo status

                                    //     }
                                    // }

                                }
                                _ => {
                                    PathType::None;
                                }
                                // EventKind::Modify(ModifyKind) => {
                                //     // If path is dir
                                //     // Check complete/incomplete tag
                                //     //     Warn if incomplete!
                                //     // else if file
                                //     // check if it is incomplete fastq file 
                                //     // if not warn!
                                // }
                                // EventKind::Remove(RemoveKind) => {},
                                //     // If path is dir
                                //     // Warn!
                                //     // else if file 
                                //     // check if it is incomplete fastq file
                                //     // if not warn!
                                // EventKind::Access(AccessKind) => {}
                                // EventKind::Any => {}
                                // EventKind::Other => {}
                                // // let data = check_event(&path, event).unwrap();
                            // // data update
                            // // mongohandiling(package, event)
                            }
                        }
                         Err(error) => {
                                log::error!("Error: {error:?}");
                        }
                    }
				}
				Message::Scan(e) => {
                    println!("Scan event {e:?}");
                },
                // data checkfile
                // mongohandling(package, scan)
			}
		}

		Ok(())
	}
    
    use super::utils_checkfile::{validate_create_dir, validate_create_file, FileInfo, FileError, DirError};
    use std::fmt::Display;
    
    #[derive(Debug)]
    pub enum PathType {
        Dir(Result<(),DirError>),
        File(Result<FileInfo,FileError>),
        None
    }

    fn handle_create_event(watched_path: &Path, mut event: notify::Event) -> PathType {
        let path = event.paths.pop().unwrap();
        if path.is_dir() {
            log::debug!("Directory of: {:?} is created", &path);
            PathType::Dir(validate_create_dir(&watched_path, &path.as_path()))
        } else {
            log::debug!("File of: {:?} is created", &path);
            PathType::File(validate_create_file(&watched_path, &path.as_ref()))
        }
    }

    // fn handle_modify_event<P: AsRef<Path>>(watched_path: P, mut event: notify::Event) {
    //     let path = event.paths.pop().unwrap();
    //     if path.is_dir() {
    //         // check dir not mark as finish
    //         // ignore if not
    //         validate_modify_dir(watched_path, path);
    //     } else {
    //         // check parent dir not mark as finish
    //         // check for only fastq 
    //         // warn if not 
    //         validate_modify_file(watched_path, &path);
    //     }
    // }

    // fn handle_remove_event<P: AsRef<Path>>(watched_path: P, mut event: notify::Event) {
    //     let path = event.paths.pop().unwrap();
    //     if path.is_dir() {
    //         // check dir not mark as finish
    //         // ignore if not
    //         validate_remove_dir(watched_path, path);
    //     } else {
    //         // check parent dir not mark as finish
    //         // check for only fastq 
    //         // warn if not 
    //         validate_remove_file(watched_path, &path);
    //     }
    // }

   
}




mod utils_checkfile {

    use std::fs::File;
    use std::io::{self, Read, BufRead};
	use std::path::{Path, PathBuf};
    use regex::Captures;
    use regex::{Regex, RegexSet};
    use md5::{Md5, Digest};
	use hex::ToHex;
    use std::sync::mpsc::channel;
    use std::thread;

    #[derive(Clone,Debug)]
    pub enum FileType {
        FQ(i32, FileStatus),
        MD5_1(i32),
        MD5_2(i32, Option<String>),
        FQSTAT(i32),
        Report
    }

    #[derive(Clone, Copy, Debug)]
    pub enum FileStatus {
        Finish,
        Pending
    }

    #[derive(Debug)]
    pub enum FileError {
        ParentDirIsRoot(String),
        ParentDirIsSubdir(String),
        UnknownExtension(String),
        FilePatternError(PatternError),
        PatternConflict(String),
            // ROOT 
            // INCORRECT DIR
    }

    #[derive(Debug)]
    pub enum PatternError {
        IdError(String),
        FlowcellIdError(String),
        FlowcellIdxError(String),
        CustomerIdError(String),
        CustomerIdxError(String),
        LaneIdError(String),
        DateYearError(String),
        DateMonthError(String),
        DateDayError(String),
        Others(String)
    }

    #[derive(Debug)]
    pub enum DirError {
        NonRootDirError(String),
        DirPatternError(PatternError),
    }


    pub fn validate_create_dir<P: AsRef<Path>>(watched_path: P, path: P) -> Result<(),DirError> {
        /* 
         * Function take watched path and event.paths from notify::Event crate 
         * Function return 
         *      Ok - Dir struct for GET/POST
         *      Err - if any validation steps fail.
         * Function check for 2 things:
         *  - First is validating the path of directory, the path must not
         *    be anywhere except at the watched directory
         *  - Second is validating the pattern of directory, the pattern must 
         *    consist of all following section:
         *
         *    Example: D2303_E200007213_20231024
         *
         *      - Thaiomic ID 
         *      - Flowcell ID
         *      - Timestamp (YYYYMMDD)
         *
         * TODO: catch following pattern
         *          - Err subfolder 
         *          - Err wrong pattern
         * 
         */        
            // Subfolder check
            let parent_dir = path.as_ref().parent().unwrap();
            if parent_dir != watched_path.as_ref() {
                log::warn!("Directory {:?} is not create on watched directory", path.as_ref().display().to_string());
                Err(DirError::NonRootDirError(path.as_ref().display().to_string()))
            } else {

            // get dir pattern
                match validate_pattern_dir(&path.as_ref().display().to_string()) {
                    Ok(captures) => { 
                        log::debug!("Succecfully validate directory pattern {:?}", captures.get(0).map_or("", |m| m.as_str())); 
                        Ok(())
                        // TODO: GET
                        // TODO: POST
                        // TODO: Get status
                        },
                    Err(vec_err) => {
                        // TODO: handle each err in vector
                        log::warn!("Failed valaidating directory pattern of {:?}", path.as_ref().display().to_string());
                        log::warn!("{:?}", vec_err);
                        Err(DirError::DirPatternError(PatternError::Others(path.as_ref().display().to_string())))
                    }
                }
            }
    }
	

    #[derive(Debug)]
    pub struct FileInfo {
        parent_dir: String,
		basename: String,
        thaiomic_id: Option<String>,
        customer_id: Option<String>,
        customer_idx: Option<String>,
        flowcell_id: Option<String>,
        flowcell_idx: Option<String>,
        lane_id: Option<String>,
        thaiomic_code: Option<String>,
		mate: i32,
        file_type: FileType,
        file_path: String,
        md5: Option<String>,
        status: Option<String>
    }

	impl FileInfo {
		// Define your new function here
		fn new(parent_dir: String, basename: String, thaiomic_id: Option<String>, customer_id: Option<String>, customer_idx: Option<String>, flowcell_id: Option<String>, flowcell_idx: Option<String>, lane_id: Option<String>, thaiomic_code: Option<String>, mate: i32, file_type: FileType, file_path: String, md5: Option<String>, status: Option<String>) -> Self {
			FileInfo {
				parent_dir,
				basename,
				thaiomic_id,
				customer_id,
				customer_idx,
				flowcell_id,
				flowcell_idx,
				lane_id,
                thaiomic_code,
				mate,
				file_type,
				file_path,
				md5,
				status,
			}
		}

		// Example of another function
		// fn display_info(&self) {
		// 	println!("File Type: {:?}", self.file_type);
		// 	println!("File Path: {:?}", self.file_path);
		// 	println!("MD5: {:?}", self.md5);
		// 	println!("Status: {:?}", self.status);
		// }
	}


    pub fn validate_create_file<P: AsRef<Path>>(watched_path: P, path: P) -> Result<FileInfo, FileError> {

        /*
         * Function take watched path and event.paths from notify::Event crate 
         * Function return 
         *      Ok() - FileInfo struct for GET/POST.
         *      Err() - If one of validtion fail.
         * Function check for 3 things:
         *  - First is validating the path of directory, the path must 
         *    be only at two level under watching directory.
         *  - Second is validating the pattern of directory, the pattern must 
         *    consist of all following section:
         *  - Third is that the pattern must match with its parent directory.
         *
         *    Example:
         *
         *    T2302_81220513400257_RTC_56-E250004334_L01_56_1
         *    |            |            |   |        |   |  |
         *    ThaiOmicID   |            CustomerIDx  |   |  Mate
         *                 CustomerID   |   |        |   FlowcellIDx
         *                              FlowcellID LaneID
         *                              |
         *                              ThaiOmicCode
         *
         *
         * TODO: ignore if file in more than 1 level under watched folder
         */
            let parent_dir = path.as_ref().parent().unwrap();
            log::debug!("Check if file is created on watched directory...");
            if format!("{}/",parent_dir.display().to_string()) ==  watched_path.as_ref().display().to_string() {
                log::warn!("File {:?} is create on a watched folder {:?}, file is ignored...", path.as_ref().display().to_string(), parent_dir);
                Err(FileError::ParentDirIsRoot(path.as_ref().display().to_string()))
            } else {
                // Determine parent dir 
				log::debug!("Getting pattern of file's parent directory...");
                match validate_pattern_dir(&parent_dir.to_string_lossy().to_string()) {
                    Ok(dir_captures) => { 
                        log::debug!("Succecfully validate file's parent directory pattern {:?}", dir_captures.get(0).map_or("", |m| m.as_str())); 
                        log::info!("File's parent dir: {:?}", dir_captures.get(0).map_or("", |m| m.as_str())); 
						let (dir_basename, [thaiomic_id, flowcell_id, year, month, day]) = dir_captures.extract();

                        log::debug!("Getting file's pattern..");
                        match validate_pattern_file(&path.as_ref().display().to_string()){
                            Ok(file_captures) => {
                                log::debug!("Succecfully validate file pattern {:?}", file_captures.get(0).map_or("", |m| m.as_str())); 
                                log::info!("Verify file's basename: {:?}", file_captures.get(0).map_or("", |m| m.as_str())); 

                                log::debug!("Checking file and its parent dir pattern...");
                                if file_captures.name("thaiomic_id").map_or("", |m| m.as_str()) != dir_captures.name("thaiomic_id").map_or("", |m| m.as_str()) {
									log::warn!("File and Dir Pattern not matched: thaiomic_id");
                                    return Err(FileError::PatternConflict(path.as_ref().display().to_string()))
                                }
                                if file_captures.name("flowcell_id").map_or("", |m| m.as_str()) != dir_captures.name("flowcell_id").map_or("", |m| m.as_str()) {
									log::warn!("File and Dir Pattern not matched: flowcell_id");
                                    return Err(FileError::PatternConflict(path.as_ref().display().to_string()))
                                }
                                log::debug!("File and its dir pattern matched!");

                                // TODO: get pattern out of file_captures
								let (file_basename, [temp, thaiomic_id, customer_id, thaiomic_code, customer_idx, flowcell_id, lane_id, flowcell_idx]) = file_captures.extract();

                                // TODO: match file and dir pattern 
                                // log::info!("{:?} is created on watched directory", file_captures.get(0).map_or("", |m| m.as_str())); 
                                log::debug!("Determine file extension...");
                                if let Ok(ext) = get_extension(&path.as_ref().display().to_string()) {
                                    // TODO: GET
                                    // TODO: POST
                                    // TODO: Get status
                                    let filetype = ext.clone();
                                    match ext {
                                        FileType::MD5_1(mate) => {
                                            log::debug!("[{}]: MD5 (type 1) of FQ_{} found!", &file_basename, &mate);
                                            let md5_hash = read_md5_from_file(&path.as_ref().display().to_string()).unwrap();
                                            let file_info = FileInfo::new(
												dir_basename.to_string(),
												file_basename.to_string(),
												wrap_string(thaiomic_id),
												wrap_string(customer_id),
												wrap_string(customer_idx),
												wrap_string(flowcell_id),
												wrap_string(flowcell_idx),
												wrap_string(lane_id),
                                                wrap_string(thaiomic_code),
												mate,
												filetype, 
												path.as_ref().display().to_string(), 
												Some(md5_hash), 
												None
											);
											log::debug!("{:?}", file_info);
                                            Ok(file_info)
                                        }
                                        FileType::MD5_2(mate, md5_hash) => {
                                            // TODO: handle md5 not found error
                                            log::debug!("[{}]: MD5 (type 2) of FQ_{} found!", &file_basename, mate);
                                            if let Some(hash) = md5_hash {
                                                let file_info = FileInfo::new(
                                                    dir_basename.to_string(),
                                                    file_basename.to_string(),
                                                    wrap_string(thaiomic_id),
                                                    wrap_string(customer_id),
                                                    wrap_string(customer_idx),
                                                    wrap_string(flowcell_id),
                                                    wrap_string(flowcell_idx),
                                                    wrap_string(lane_id),
                                                    wrap_string(thaiomic_code),
                                                    mate,
                                                    filetype, 
                                                    path.as_ref().display().to_string(), 
                                                    Some(hash.to_string()), 
                                                    None
                                                );
												log::debug!("{:?}", file_info);
                                                Ok(file_info)
                                                
                                            } else {
                                                log::warn!("[{}]: {:?} md5 type2 not contain md5", &file_basename, &path.as_ref().display().to_string());
                                                log::warn!("Try to read first line of the file..");
                                                let md5_hash = read_md5_from_file(&path.as_ref().display().to_string()).unwrap();
                                                let file_info = FileInfo::new(
                                                    dir_basename.to_string(),
                                                    file_basename.to_string(),
                                                    wrap_string(thaiomic_id),
                                                    wrap_string(customer_id),
                                                    wrap_string(customer_idx),
                                                    wrap_string(flowcell_id),
                                                    wrap_string(flowcell_idx),
                                                    wrap_string(lane_id),
                                                    wrap_string(thaiomic_code),
                                                    mate,
                                                    filetype, 
                                                    path.as_ref().display().to_string(), 
                                                    Some(md5_hash), 
                                                    None
                                                );
												log::debug!("{:?}", file_info);
                                                Ok(file_info)
                                            }
                                        }
                                        FileType::FQ(mate, status) => {
                                            match status {
                                                FileStatus::Pending => {
                                                log::debug!("[{}]: Pending FQ_{} found!", &file_basename, &mate);
                                                    let file_info = FileInfo::new(
                                                        dir_basename.to_string(),
                                                        file_basename.to_string(),
                                                        wrap_string(thaiomic_id),
                                                        wrap_string(customer_id),
                                                        wrap_string(customer_idx),
                                                        wrap_string(flowcell_id),
                                                        wrap_string(flowcell_idx),
                                                        wrap_string(lane_id),
                                                        wrap_string(thaiomic_code),
                                                        mate,
                                                        filetype, 
                                                        path.as_ref().display().to_string(), 
                                                        None, 
                                                        Some(String::from("Pending"))
                                                    );
													log::debug!("{:?}", file_info);
                                                    Ok(file_info)
                                                }
                                                FileStatus::Finish => {
                                                log::debug!("[{}]: Finish FQ_{} found!", &file_basename, &mate);
                                                    let md5_hash = calculate_md5(&path.as_ref().display().to_string());
                                                    let file_info = FileInfo::new(
                                                        dir_basename.to_string(),
                                                        file_basename.to_string(),
                                                        wrap_string(thaiomic_id),
                                                        wrap_string(customer_id),
                                                        wrap_string(customer_idx),
                                                        wrap_string(flowcell_id),
                                                        wrap_string(flowcell_idx),
                                                        wrap_string(lane_id),
                                                        wrap_string(thaiomic_code),
                                                        mate,
                                                        filetype,
                                                        path.as_ref().display().to_string(),
                                                        Some(md5_hash.unwrap()),
                                                        Some(String::from("Finish"))
                                                    );
													log::debug!("{:?}", file_info);
                                                    Ok(file_info)
                                                }
                                            }
                                        }
										FileType::FQSTAT(mate) => {
                                            log::debug!("[{}]: FQSTAT_{} found!", &file_basename, &mate);
											let file_info = FileInfo::new(
                                                dir_basename.to_string(),
                                                file_basename.to_string(),
                                                wrap_string(thaiomic_id),
                                                wrap_string(customer_id),
                                                wrap_string(customer_idx),
                                                wrap_string(flowcell_id),
                                                wrap_string(flowcell_idx),
                                                wrap_string(lane_id),
                                                wrap_string(thaiomic_code),
                                                mate,
                                                filetype,
                                                path.as_ref().display().to_string(),
                                                None,
                                                None
                                            );
											log::debug!("{:?}", file_info);
                                            Ok(file_info)
										}
										FileType::Report => { 
                                            log::debug!("[{}]: REPORT found!", &file_basename);
											let file_info = FileInfo::new(
                                                dir_basename.to_string(),
                                                file_basename.to_string(),
                                                wrap_string(thaiomic_id),
                                                wrap_string(customer_id),
                                                wrap_string(customer_idx),
                                                wrap_string(flowcell_id),
                                                wrap_string(flowcell_idx),
                                                wrap_string(lane_id),
                                                wrap_string(thaiomic_code),
                                                0,
                                                filetype,
                                                path.as_ref().display().to_string(),
                                                None,
                                                None
                                            );
											log::debug!("{:?}", file_info);
                                            Ok(file_info)
										}
										
                                    }
                                } else {
                                    Err(FileError::PatternConflict(path.as_ref().display().to_string()))

                                }
                            }
                            Err(file_vec_err) => {
                                // TODO: handle each err in vector
                                log::warn!("validate_file_pattern_error {:?}", &path.as_ref().display().to_string());
								for err in file_vec_err.into_iter() {
									log::warn!("{:?}",err);
								}
                                Err(FileError::FilePatternError(PatternError::Others(path.as_ref().display().to_string())))

                            }
                        
                        }
                    },
                    Err(dir_vec_err) => {
                        // TODO: handle each err in vector
                        log::warn!("validate_dir_pattern_error {:?}", &path.as_ref().display().to_string());
						for err in dir_vec_err.into_iter() {
							log::warn!("{:?}",err);
						}
                        Err(FileError::FilePatternError(PatternError::Others(path.as_ref().display().to_string())))
                    }
                }

            }
                
    }

    fn get_extension(path: &str) -> Result<FileType,FileError>{
            // set of regex to cactch all extension
            //              fastq_1             - 1.fq.gz
            //              incomplete fastq_1  - 1.fq.gz.x23f21
            //              fastq_2             - 2.fq.gz
            //              incomplete fastq_2  - 2.fq.gz.gqe21w
            //              md5_1 type-1        - 1.fq.gz.md5.txt
            //              md5_1 type-2        - 1.fq.gz.d875fa91b8273ee74667dccd3c4db520.md5.txt
            //              md5_2 type-1        - 2.fq.gz.md5.txt
            //              md5_2 type-2        - 2.fq.gz.a624167802093729255092719181a679.md5.txt
            //              fqstat_1            - 1.fq.fqStat.txt
            //              fqstat_2            - 2.fq.fqStat.txt
            //              report              - report.html

            let set = [r"(?<mate>[12])(?<ext>\.fq\.gz)(?<hash>.?\w{0,6})$",
                 r"(?<mate>[12])(?<ext>\.fq\.gz\.md5\.txt)$",
                 r"(?<mate>[12])(?<ext>\.fq\.gz).(?<md5>\w{0,32})(?<ext2>\.md5\.txt)$",
                 r"(?<mate>[12])(?<ext>\.fq\.fqStat\.txt)$",
                 r"(?<ext>\.report\.html)$"];
            let re_set = RegexSet::new(&set).unwrap();

            if re_set.is_match(path) {
                // 0 = FQ
                // 1 = md5-1
                // 2 = md5-2
                // 3 = fqstat
                // 4 = report
               let ext_matches: Vec<_> = re_set.matches(&path).into_iter().collect();
               if ext_matches.len() > 1 {
                   log::warn!("{:?}: match more than one pattern", &path);
                   log::debug!("{:?}", ext_matches);
                   Err(FileError::UnknownExtension(String::from("More than one patterns")))
               } else {
                   match ext_matches[0] {
                       0 => {
                            let re_fq = Regex::new(set[0]).unwrap();
                            let matches_fq = re_fq.captures(&path).unwrap();
                            let mate = matches_fq.name("mate").map_or("", |m| m.as_str());
                            if let Some(hash) = matches_fq.name("hash").map( |m| m.as_str() ) {
                                if hash == "" {
                                    // log::debug!("{:?} - hash : {}", &path.to_string(), hash);
                                    let file_status = FileStatus::Finish;

                                    return Ok(FileType::FQ(mate.parse().unwrap(), file_status))
                                } else {
                                    // log::debug!("{:?} - hash : {}", &path.to_string(), hash);
                                    let file_status = FileStatus::Pending;
                                    return Ok(FileType::FQ(mate.parse().unwrap(), file_status))
                                }
                            } else {
                                Err(FileError::UnknownExtension(String::from("hash group do no match")))
                            }
                       },
                       1 => {
                           let re_md5_1 = Regex::new(set[1]).unwrap();
                           let matches_md5_1 = re_md5_1.captures(&path).unwrap();
                           let mate = matches_md5_1.name("mate").map_or("", |m| m.as_str());
                           Ok(FileType::MD5_1(mate.parse().unwrap()))
                       },
                       2 => {
                           let re_md5_2 = Regex::new(set[2]).unwrap();
                           let matches_md5_2 = re_md5_2.captures(&path).unwrap();
                           let mate = matches_md5_2.name("mate").map_or("", |m| m.as_str());
                           let md5 = matches_md5_2.name("md5").map_or("", |m| m.as_str());
                           Ok(FileType::MD5_2(mate.parse().unwrap(), Some(md5.to_string())))
                       },
                       3 => {
                           let re_fqstat = Regex::new(set[3]).unwrap();
						   // log::debug!("{:?}",&path.to_string());
                           let matches_fqstat = re_fqstat.captures(&path).unwrap();
                           let mate = matches_fqstat.name("mate").map_or("", |m| m.as_str());
                           Ok(FileType::FQSTAT(mate.parse().unwrap()))
                       },
                       4 => {
                           let re_report = Regex::new(set[4]).unwrap();
                           let matches_fqstat = re_report.captures(&path).unwrap();
                           Ok(FileType::Report)
                       }
                       _ => {
                           Err(FileError::UnknownExtension(path.to_string()))
                       }
                   }
               } 
               
            } else {
                log::warn!("{:?}: Unknown extension", &path);
                Err(FileError::UnknownExtension(path.to_string()))
            }
    }

    fn validate_pattern_dir(path: &str) -> Result<regex::Captures,Vec<PatternError>>{
        /*
         * Function take string of path and Regex pattern
         * Function return either regex::Capture result or vector of err
         */
        let dir_pattern = Regex::new(r"(?<thaiomic_id>[DT][0-9]+)_(?<flowcell_id>E[0-9]+[A-Z]*)_(?<year>20[0-9][0-9])(?<month>[01][0-9])(?<day>[0123][0-9])").unwrap();
        let mut errors = Vec::new();

        log::debug!("Start validating directory pattern of {}", &path);

        if let Some(matched) = dir_pattern.captures(path) {

            if let None = matched.name("thaiomic_id") {
                errors.push(PatternError::IdError(path.to_string()));
            }
            if let None = matched.name("flowcell_id") {
                errors.push(PatternError::FlowcellIdError(path.to_string()));
            }
            if let None = matched.name("year") {
                errors.push(PatternError::DateYearError(path.to_string()));
            }
            if let None = matched.name("month") {
                errors.push(PatternError::DateMonthError(path.to_string()));
            }
            if let None = matched.name("day") {
                errors.push(PatternError::DateDayError(path.to_string()));
            }
            
        } else {
            errors.push(PatternError::Others(path.to_string()));
        }

        if !errors.is_empty() {
            Err(errors)
        } else {
			Ok(dir_pattern.captures(path).unwrap())

        }

    }

    fn validate_pattern_file(path: &str) -> Result<regex::Captures,Vec<PatternError>>{
        /*
         * Function take string of path and Regex pattern
         * Function return either regex::Capture result or vector of err
         */
        let file_pattern = Regex::new(r"(?<tmp>\.?)(?<thaiomic_id>[DT][0-9]+)_(?<customer_id>[0-9]+)_?(?<thaiomic_code>[A-Z]*)_?_(?<customer_idx>[0-9]{2})-(?<flowcell_id>E[0-9]+[A-Z]*)_(?<lane_id>L[0-9]{2})_(?<flowcell_idx>[0-9]+)").unwrap();

        let mut errors = Vec::new();
        if let Some(matched) = file_pattern.captures(path) {

            if let None = matched.name("thaiomic_id"){
                errors.push(PatternError::IdError(path.to_string()));
            }
            if let None = matched.name("customer_id") {
                errors.push(PatternError::CustomerIdError(path.to_string()));
            }
            if let None = matched.name("customer_idx") {
                errors.push(PatternError::CustomerIdxError(path.to_string()));
            }
            if let None = matched.name("flowcell_id") {
                errors.push(PatternError::FlowcellIdError(path.to_string()));
            }
            if let None = matched.name("lane_id") {
                errors.push(PatternError::LaneIdError(path.to_string()));
            }
            if let None = matched.name("flowcell_idx") {
                errors.push(PatternError::FlowcellIdxError(path.to_string()));
            }
        } else {
            errors.push(PatternError::Others(path.to_string()));
        }

        if !errors.is_empty() {
            Err(errors)
        } else {
			Ok(file_pattern.captures(path).unwrap())
        }
    }

	fn read_md5_from_file(md5_file_path: &str) -> io::Result<String> {
        log::debug!("Reading first line of {}", md5_file_path);
		let mut md5_hash = String::new();
		let file = File::open(md5_file_path)?;
		let mut reader = io::BufReader::new(file);

		// Read the MD5 hash from the file
		reader.read_line(&mut md5_hash)?;

		// Trim any leading or trailing whitespace
		md5_hash = md5_hash.trim().to_string();

		Ok(md5_hash)
	}

    fn calculate_md5(file_path: &str) -> io::Result<String> {
        log::debug!("Start MD5CheckSum on {}: ", file_path);
		// let mut file = File::open(file_path)?;
		let mut file = File::open(file_path)?;
		let mut buffer = Vec::new();

		// Read the entire file into a buffer
		file.read_to_end(&mut buffer)?;

		// Calculate the MD5 hash of the file content
		let mut md5 = Md5::new();
		md5.update(&buffer);
		let result = md5.finalize();

		// Convert the MD5 hash to a hexadecimal string
        let hex_string = hex::encode(result);
        log::debug!("Finish MD5CheckSum on {}: {}", file_path, hex_string);

		Ok(hex_string)
	 } 
    
    fn wrap_string(txt: &str) -> Option<String> {
        if txt.is_empty() {
            None
        } else {
            Some(txt.to_string())
        }
        
    }
    

}
