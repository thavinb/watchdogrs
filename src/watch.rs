pub mod utils_checkfile {

    use std::fs::{File, self};
    use std::io::{self, Read, BufRead};
	use std::path::{Path, PathBuf};
    use regex::Captures;
    use regex::{Regex, RegexSet};
    use md5::{Md5, Digest};
	use hex::ToHex;
    use std::sync::mpsc::channel;
    use std::thread;
    use human_bytes::human_bytes;

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
        ModifyFinishFile(String),
        RemoveFinishFile(String)
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
        ModifyFinishDir(String),
        RemoveDir(String),
    }

    #[derive(Debug)]
    pub enum PathType {
        Dir(Result<(),DirError>),
        File(Result<FileInfo,FileError>),
        None
    }

    pub fn handle_create_event(watched_path: &Path, mut event: notify::Event) -> PathType {
        let path = event.paths.pop().unwrap();
        if path.is_dir() {
            log::debug!("Directory of: {:?} is created", &path);
            PathType::Dir(validate_create_dir(&watched_path, &path.as_path()))
        } else {
            log::debug!("File of: {:?} is created", &path);
            PathType::File(validate_create_file(&watched_path, &path.as_ref()))
        }
    }

    pub fn handle_modify_event(watched_path: &Path, mut event: notify::Event) -> PathType {
        let path = event.paths.pop().unwrap();
        if path.is_dir() {
            // TODO: Warn if dir path should not be modify(Finish).
            // TODO: Fetch db to confirm
            PathType::Dir(Err(DirError::ModifyFinishDir(path.display().to_string())))
        } else {
            PathType::File(validate_modify_file(watched_path, &path.as_ref()))
        }
    }

    pub fn handle_remove_event(watched_path: &Path, mut event: notify::Event) -> PathType {
        let path = event.paths.pop().unwrap();
        if path.is_dir() {
            // check dir not mark as finish
            // ignore if not
            PathType::Dir(Err(DirError::RemoveDir(path.display().to_string())))
        } else {
            // check parent dir not mark as finish
            // check for only fastq 
            // warn if not 
            if let Err(e) = validate_remove_file(watched_path, &path.as_ref()) {
                PathType::File(Err(e))
            } else {
                PathType::None
            }
        }
    }
    fn validate_create_dir<P: AsRef<Path>>(watched_path: P, path: P) -> Result<(),DirError> {
        /* 
         * Function take watched path and event.paths from notify::Event crate 
         * Function return 
         *      Ok - Dir struct for GET/POST
         *      Err - if any validation steps fail.
         * Function validate 2 things:
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
        file_size: u64,
        md5: Option<String>,
        status: Option<String>
    }

	impl FileInfo {
		// Define your new function here
		fn new(
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
            file_size: u64,
            md5: Option<String>, 
            status: Option<String>
        ) -> Self {
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
                file_size,
				md5,
				status,
			}
		}

		// Pretty print struct
		// fn display_info(&self) {
		// 	println!("File Type: {:?}", self.file_type);
		// 	println!("File Path: {:?}", self.file_path);
		// 	println!("MD5: {:?}", self.md5);
		// 	println!("Status: {:?}", self.status);
		// }
	}

    pub fn validate_modify_file<P: AsRef<Path>>(watched_path: P, path: P) -> Result<FileInfo, FileError> {
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

								let (file_basename, [temp, thaiomic_id, custom_name, customer_id, thaiomic_code, customer_idx, flowcell_id, lane_id, flowcell_idx]) = file_captures.extract();

                                // TODO: match file and dir pattern 
                                // log::info!("{:?} is created on watched directory", file_captures.get(0).map_or("", |m| m.as_str())); 
                                log::debug!("Determine file extension...");
                                if let Ok(ext) = get_extension(&path.as_ref().display().to_string()) {
                                    // TODO: GET
                                    // TODO: POST
                                    // TODO: Get status
                                    let filetype = ext.clone();
                                    match ext {
                                        FileType::FQ(mate, status) => {
                                            match status {
                                                FileStatus::Pending => {
                                                log::debug!("[{}]: Pending FQ_{} found!", &file_basename, &mate);
                                                log::debug!("[{}] - Starting temporary FQSTAT_{} File size: {:?}", &file_basename, &mate, human_bytes(fs::metadata(&path).unwrap().len() as f64));
                                                    let file_info = FileInfo::new(
                                                        dir_basename.to_string(),
                                                        String::from(&file_basename[1..].to_string()), // basename expected to be hidden - ".D2301_L01_56"
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
                                                        fs::metadata(&path).unwrap().len(),
                                                        None, 
                                                        Some(String::from("Pending"))
                                                    );
													log::debug!("{:?}", file_info);
                                                    Ok(file_info)
                                                }
                                                FileStatus::Finish => {
                                                log::warn!("[{}]: Written on finish FQ_{} file!", &file_basename, &mate);
                                                Err(FileError::ModifyFinishFile(path.as_ref().display().to_string()))
                                                }
                                            }
                                        }
                                        _ => {
                                            Err(FileError::ModifyFinishFile(path.as_ref().display().to_string()))
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
         *    T2302_NAME_81220513400257_RTC_56-E250004334_L01_56_1
         *    |     |      |            |   |        |   |  |
         *    ThaiOmicID   |            CustomerIDx  |   |  Mate
         *          |      CustomerID   |   |        |   FlowcellIDx
         *          CustomName        FlowcellID   LaneID
         *                              |
         *                              ThaiOmicCode
         *
         *
         * 
         */
        // TODO: ignore if file in more than 1 level under watched folder
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

								let (file_basename, [temp, thaiomic_id, custom_name, customer_id, thaiomic_code, customer_idx, flowcell_id, lane_id, flowcell_idx]) = file_captures.extract();

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
                                            log::debug!("[{}] - MD5 (type 1) of FQ_{} File size: {:?}", &file_basename, &mate, human_bytes(fs::metadata(&path).unwrap().len() as f64));
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
                                                fs::metadata(&path).unwrap().len(),
												Some(md5_hash), 
												None
											);
											log::debug!("{:?}", file_info);
                                            Ok(file_info)
                                        }
                                        FileType::MD5_2(mate, md5_hash) => {
                                            // TODO: handle md5 not found error
                                            log::debug!("[{}]: MD5 (type 2) of FQ_{} found!", &file_basename, mate);
                                            log::debug!("[{}] - MD5 (type 2) of FQ_{} File size: {:?}", &file_basename, &mate, human_bytes(fs::metadata(&path).unwrap().len() as f64));
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
                                                    fs::metadata(&path).unwrap().len(),
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
                                                    fs::metadata(&path).unwrap().len(),
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
                                                log::debug!("[{}] - Starting temporary FQSTAT_{} File size: {:?}", &file_basename, &mate, human_bytes(fs::metadata(&path).unwrap().len() as f64));
                                                    let file_info = FileInfo::new(
                                                        dir_basename.to_string(),
                                                        String::from(&file_basename[1..].to_string()), // basename expected to be hidden - ".D2301_L01_56"
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
                                                        fs::metadata(&path).unwrap().len(),
                                                        None, 
                                                        Some(String::from("Pending"))
                                                    );
													log::debug!("{:?}", file_info);
                                                    Ok(file_info)
                                                }
                                                FileStatus::Finish => {
                                                log::debug!("[{}]: Finish FQ_{} found!", &file_basename, &mate);
                                                log::debug!("[{}] - Finish FQ_{} File size: {:?}", &file_basename, &mate, human_bytes(fs::metadata(&path).unwrap().len() as f64));
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
                                                        fs::metadata(&path).unwrap().len(),
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
                                            log::debug!("[{}] - FQSTAT_{} File size: {:?}", &file_basename, &mate, human_bytes(fs::metadata(&path).unwrap().len() as f64));
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
                                                fs::metadata(&path).unwrap().len(),
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
                                                fs::metadata(&path).unwrap().len(),
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
    pub fn validate_remove_file<P: AsRef<Path>>(watched_path: P, path: P) -> Result<(),FileError> {
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

								let (file_basename, [temp, thaiomic_id, custom_name, customer_id, thaiomic_code, customer_idx, flowcell_id, lane_id, flowcell_idx]) = file_captures.extract();

                                // TODO: match file and dir pattern 
                                // log::info!("{:?} is created on watched directory", file_captures.get(0).map_or("", |m| m.as_str())); 
                                log::debug!("Determine file extension...");
                                if let Ok(ext) = get_extension(&path.as_ref().display().to_string()) {
                                    // TODO: GET
                                    // TODO: POST
                                    // TODO: Get status
                                    let filetype = ext.clone();
                                    match ext {
                                        FileType::FQ(mate, status) => {
                                            match status {
                                                FileStatus::Pending => {
                                                    log::debug!("[{}]: Pending FQ_{} is removed expect Finished FQ_{}!", &file_basename, &mate, &mate);
                                                    //TODO: Check file create somehow
                                                    Ok(())
                                                }
                                                FileStatus::Finish => {
                                                    log::warn!("[{}]: Remove finish FQ_{} file!", &file_basename, &mate);
                                                    Err(FileError::RemoveFinishFile(path.as_ref().display().to_string()))
                                                }
                                            }
                                        }
                                        _ => {
                                            Err(FileError::RemoveFinishFile(path.as_ref().display().to_string()))
                                        }
                                    }                                    
                                } else {
                                    Err(FileError::RemoveFinishFile(path.as_ref().display().to_string()))
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
        let file_pattern = Regex::new(r"(?<tmp>\.?)(?<thaiomic_id>[DT][0-9]+)_(?<custom_name>[0-9AZ]*)_(?<customer_id>[0-9]+)_?(?<thaiomic_code>[A-Z]*)_?_(?<customer_idx>[0-9]{2})-(?<flowcell_id>E[0-9]+[A-Z]*)_(?<lane_id>L[0-9]{2})_(?<flowcell_idx>[0-9]+)").unwrap();

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
