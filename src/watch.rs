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

		for res in rx {
			match res {
				Message::Event(e) => {
                    match e {
                        Ok(event) => check_event(&path, event),
                        Err(error) => log::error!("Error: {error:?}")
                    }
				},
				Message::Scan(e) => println!("Scan event {e:?}"),
			}
		}

		Ok(())
	}

    enum UploadStatus {
        Done(File),
        Uploading(File),
        Missing
    }

    enum ChecksumStatus  {
        Passed(Packages),
        Failed(Packages)
    }
    
    enum Packages { 
        Fastq1(std::path::PathBuf),
        Fastq2(std::path::PathBuf),
        Fqstat1(std::path::PathBuf),
        Fqstat2(std::path::PathBuf),
        Md51(std::path::PathBuf),
        Md52(std::path::PathBuf),
        Report(std::path::PathBuf)
    }

    struct File { 
        sampleID : u8,
        file: Packages,
    }

    struct SampleList {
        sampleID : String,
        basename : String,
        dirpath : String,
        fastq_1 : UploadStatus,
        fastq_2 : UploadStatus,
        fqstat_1 : UploadStatus,
        fqstat_2 : UploadStatus,
        md5_1 : UploadStatus,
        md5_2 : UploadStatus,
        report : UploadStatus
    }

    impl SampleList {
        fn new(
            sampleID : String,
            dirpath : String,
        ) -> Self {
            Self { 
                sampleID: String::from("non"),
                basename: String::from("non"),
                dirpath,
                fastq_1: UploadStatus::Missing,
                fastq_2: UploadStatus::Missing,
                fqstat_1: UploadStatus::Missing,
                fqstat_2: UploadStatus::Missing,
                md5_1: UploadStatus::Missing,
                md5_2: UploadStatus::Missing,
                report: UploadStatus::Missing
            }

        } 
        fn validate(){

        }
        fn checksum(){

        }

    }

    struct Basnename{
        ThaiOmicID: String,
        CustomerID: String,
        CustomerIndex: u8,
        FlowcellID: String,
        LaneID: String,
        FlowcellIndex: u8
    }

    // get event and path 
    // make data structure for event and path 
    fn check_event<P: AsRef<Path>>(watched_path: P, mut event: notify::Event){
        // let dirPattern = 
        let path = event.paths.pop().unwrap();
        // let re = Regex::new(r"(B\w+):(\w+)_([0-9]+)-(\w+)_(\w+)_(\w+)[_.](\w+)[_.](\w+)\.*(\w+)*\.*(\w+)*\.*(\w+)*").unwrap(); 
        let re_dir = Regex::new(r"(D[0-9]+)_(E[0-9]+)_(20[0-9][0-9])([01][0-9])([123][0-9])").unwrap();
        let re_basename = Regex::new(r"(?<tmp>\.?)(?<thaiomic_id>D[0-9]+)_(?<customer_id>[0-9]+)_(?<customer_index>[0-9]{2})-(?<flowcell_id>E[0-9]+)_(?<lane_id>L[0-9]{2})_(?<flowcell_index>[0-9]+)").unwrap();
        let set = [r"(1)\.(fq)\.(gz)(.?\w{0,6})$",
             r"(2)\.(fq)\.(gz)(.?\w{0,6})$",
             r"(1)\.(fq)\.(gz)\.(\w{32}).md5.txt$",
             r"(2)\.(fq)\.(gz)\.(\w{32}).md5.txt$",
             r"(1)\.(fq)\.(fqStat)\.(txt)$",
             r"(2)\.(fq)\.(fqStat)\.(txt)$",
             r"(report)\.(html)$"];
        let re_set = RegexSet::new(&set).unwrap();
        let re_fastq = Regex::new(r"([12])\.(fq)\.(gz)$").unwrap(); 
        let re_md5 = Regex::new(r"([12])\.(fq)\.(gz)\.(\w{32}).txt$").unwrap();
        let re_fqstat = Regex::new(r"([12])\.(fq)\.(fqStat)\.(txt)$").unwrap(); 
        let re_report = Regex::new(r"(report)\.(html)$").unwrap();
        let dir_path = path.parent().unwrap();
        match event.kind { 
            EventKind::Create(event) => {
                 if path.is_dir()  {
                    log::debug!("EventKind::Create(Folder) : {:?}", path);
                    let path = path.to_str().unwrap();
                    // TODO: catch all regrex error appropiately 
                    // TODO: If dir is not watched_path 
                    if dir_path != watched_path.as_ref() {
                        log::debug!("{} : {}", dir_path.to_str().unwrap(), watched_path.as_ref().to_str().unwrap());
                        log::warn!("New Directory is written into Existing directory!");
                        log::error!("Unexpected directory get uploaded as subdirectory: {}", &path);
                        // TODO: AlERT
                        panic!(r"Unexpected directory get uploaded as subdirectory!");
                    } else {
                        if let Some(dir) = extract_dir(path, re_dir) {
                            let (dir_thaiomic_id, dir_flowcell_id, dir_year, dir_month, dir_day) = dir ;
                            let id_name = format!("{}-{}", dir_thaiomic_id, dir_flowcell_id);
                            log::info!("[{}]: Directory is uploaded to watched directory!", id_name);
                            // TODO: POST in DB
                        } else {
                            log::warn!("Directory with unexpected pattern is uploaded to watched directory!: {}", &path);
                            // TODO: ALERT 
                        }
                    }
                 } else if let Some(dir) = extract_dir(dir_path.file_name().unwrap().to_str().unwrap(), re_dir) { 
                    log::debug!("EventKind::Create(File): {:?}", path);

                    // get dir path
                    let (dir_thaiomic_id, dir_flowcell_id, dir_year, dir_month, dir_day) = dir ;
                    let id_name = format!("{}-{}", dir_thaiomic_id, dir_flowcell_id);
                    
                    /* log::debug!("{:?}", re_basename.captures(path)); */
                    let path = path.to_str().unwrap();
                    // TODO: catch all regrex error appropiately
                    if let Some(basename) = re_basename.captures(path) {
                 
                        //Check tmp status
                        let tmp_status: bool = if &basename["tmp"] == "" {
                            false
                        } else {
                            log::debug!("Temporary file found: {:?}", path);
                            true
                        };

                        let thaiomic_id = &basename["thaiomic_id"];
                        let flowcell_id = &basename["flowcell_id"];
                        let customer_id = &basename["customer_id"];
                       
                        // Check if id in filename matced with dirname
                        if (dir_thaiomic_id != thaiomic_id) | (dir_flowcell_id != flowcell_id) {
                            log::warn!("Conflict found in {:?}", &path);
                            if dir_thaiomic_id != thaiomic_id {
                                log::warn!("[{}]: ThaiomicID {} do not match with directory!", id_name, thaiomic_id);
                            }
                            if dir_flowcell_id != flowcell_id {
                                log::warn!("[{}]: FlowcellID {} do not match with directory!", id_name, flowcell_id);
                            }
                        }
                        
                        // Check if suffix match with any expected pattern
                        let set_check: Vec<_> = re_set.matches(&path).into_iter().collect();
                        if set_check.len() > 1 {
                            log::error!("[{}][{}]: Filename {:?} matched with multiple pattern!", id_name, customer_id, &path);
                            for i in set_check.iter() {
                                log::error!("[{}][{}]: Pattern matched {:?}", id_name, customer_id,  set[*i]);
                            }
                        } else if set_check.len() < 1 {
                            log::error!("[{}][{}]]: Filename {:?} do not matched with any pattern!", id_name, customer_id, &path);
                        } else if set_check.len() == 1 {
                            match set_check[0] {
                                0 => {
                                    log::debug!("[{}][{}]: FWD_FQ found {:?}", id_name, customer_id, &path);
                                    if tmp_status {
                                        log::info!("[{id_name}][{customer_id}]: FWD_FQ start uploading");
                                    } else {
                                        log::info!("[{id_name}][{customer_id}]: FWD_FQ finished uploading");
                                        checksum(path.into(), "eeedc9e391d72f70f6c59bff62db4d24");
                                        // let (md5, status) = checksum(path.into(), "eeedc9e391d72f70f6c59bff62db4d24");
                                        // println!("{:?}, {:?}" , md5 , status);
                                        // TODO: MD5
                                        // TODO: POST
                                        // TODO: GET all file to check 
                                    }
                                },
                                1 => {
                                    log::debug!("[{}][{}]: RVS_FQ found {:?}", id_name, customer_id, &path);
                                    if tmp_status {
                                        log::info!("[{id_name}][{customer_id}]: RVS_FQ start uploading");
                                    } else {
                                        log::info!("[{id_name}][{customer_id}]: RVS_FQ finished uploading");
                                        // TODO: MD5
                                        // TODO: POST
                                        // TODO: GET all file to check 
                                    }
                                },
                                2 => {
                                    log::debug!("[{}][{}]: FWD_MD5 found: {:?}", id_name, customer_id, &path);
                                    let (fwd, md5) = extract_md5(path, Regex::new(set[2]).unwrap());
                                    let mate: i8 = fwd.parse().unwrap();
                                    if mate != 1 { log::error!("{} not equal 1 when it should be 1", fwd); }
                                    // TODO: Warn if tmp as file is unexpectly large.
                                    log::info!("[{id_name}][{customer_id}]: FWD_MD5 finished uploading - {md5}");
                                },
                                3 => {
                                    log::debug!("[{}][{}]: RVS_MD5 found: {:?}", id_name, customer_id, &path);
                                    let (rvs, md5) = extract_md5(path, Regex::new(set[3]).unwrap());
                                    let mate: i8 = rvs.parse().unwrap();
                                    if mate != 2 { log::error!("{} not equal 2 when it should be 2", rvs); }
                                    // TODO: Warn if tmp as file is unexpectly large.
                                    log::info!("[{id_name}][{customer_id}]: FWD_MD5 finished uploading - {md5}");
                                },
                                4 => {
                                    log::debug!("[{}][{}]: FWD_STAT found: {:?}", id_name, customer_id, &path);
                                    // TODO: Warn if tmp as file is unexpectly large.
                                    log::info!("[{id_name}][{customer_id}]: FWD_STAT finished uploading");
                                },
                                5 => {
                                    log::debug!("[{}][{}]: RVS_STAT found: {:?}", id_name, customer_id, &path);
                                    // TODO: Warn if tmp as file is unexpectly large.
                                    log::info!("[{id_name}][{customer_id}]: FWD_STAT finished uploading");
                                },
                                6 => {
                                    log::debug!("[{}][{}]: REPORT found: {:?}", id_name, customer_id, &path);
                                    // TODO: Warn if tmp as file is unexpectly large.
                                    log::info!("[{id_name}][{customer_id}]: HTML_REPORT finished uploading");
                                },
                                _ => {}
                            }
                        }
                    } else {
                        log::error!("[{}]: File {:?} not matched with expected pattern! [(?<tmp>\\.?)(?<thaiomic_id>D[0-9]+)_(?<customer_id>[0-9]+)_(?<customer_index>[0-9]{{2}})-(?<flowcell_id>E[0-9]+)_(?<lane_id>L[0-9]{{2}})_(?<flowcell_index>[0-9]+)]", id_name, &path);
                    }
                } else {
                    //dir not match 
                    // TODO: Check if root folder
                    log::warn!("File is uploaded into watched directory directly!: {:?}", &path);
                    // TODO: Following the same criteria
                }               
           },
            EventKind::Modify(event) => {
                if path.is_dir()  {
                    log::debug!("EventKind::Modify(Folder) : {:?}", path);
                    let path = path.to_str().unwrap();
                    // TODO: catch all regrex error appropiately 
                    if let Some(dir) = extract_dir(path, re_dir) {
                        let (dir_thaiomic_id, dir_flowcell_id, dir_year, dir_month, dir_day) = dir ;
                        let id_name = format!("{}-{}", dir_thaiomic_id, dir_flowcell_id);
                        log::info!("[{}]: Directory is getting update!", id_name);
                        // TODO: POST in DB
                    } else {
                        // TODO: Check if dir is root
                        // TODO: error! if not root
                        if dir_path != watched_path.as_ref() {
                            log::error!("Unknown directory is getting update!: {}", &path);
                        }
                    }
                } else if let Some(dir) = extract_dir( dir_path.file_name().unwrap().to_str().unwrap(), re_dir ) {
                    log::debug!("EventKind::Modify(File): {:?}", &path);

                    // get dir path
                    let (dir_thaiomic_id, dir_flowcell_id, dir_year, dir_month, dir_day) = dir ;
                    let id_name = format!("{}-{}", dir_thaiomic_id, dir_flowcell_id);
                    
                    /* log::debug!("{:?}", re_basename.captures(path)); */
                    let path = path.to_str().unwrap();
                    // TODO: catch all regrex error appropiately
                    if let Some(basename) = re_basename.captures(path) {
                 
                        //Check tmp status
                        let tmp_status: bool = if &basename["tmp"] == "" {
                            log::error!("Non-temporary file is getting update: {:?}, Please notify transferring procedure with sender", path);
                            false
                        } else {
                            true
                        };

                        let thaiomic_id = &basename["thaiomic_id"];
                        let flowcell_id = &basename["flowcell_id"];
                        let customer_id = &basename["customer_id"];
                       
                        // Check if id in filename matced with dirname
                        if (dir_thaiomic_id != thaiomic_id) | (dir_flowcell_id != flowcell_id) {
                            log::warn!("Conflict found in {:?}", &path);
                            if dir_thaiomic_id != thaiomic_id {
                                log::warn!("[{}]: ThaiomicID {} do not match with directory!", id_name, thaiomic_id);
                            }
                            if dir_flowcell_id != flowcell_id {
                                log::warn!("[{}]: FlowcellID {} do not match with directory!", id_name, flowcell_id);
                            }
                        }
                        
                        // Check if suffix match with any expected pattern
                        let set_check: Vec<_> = re_set.matches(&path).into_iter().collect();
                        if set_check.len() > 1 {
                            log::error!("[{}][{}]: Filename {:?} matched with multiple pattern!", id_name, customer_id, &path);
                            for i in set_check.iter() {
                                log::error!("[{}][{}]: Pattern matched {:?}", id_name, customer_id,  set[*i]);
                            }
                        } else if set_check.len() < 1 {
                            log::error!("[{}][{}]]: Filename {:?} do not matched with any pattern!", id_name, customer_id, &path);
                        } else if set_check.len() == 1 {
                            match set_check[0] {
                                0 => {
                                    log::debug!("[{}][{}]: FWD_FQ is writing {:?}", id_name, customer_id, &path);
                                    let meta = fs::metadata(&path).unwrap();
                                    let filesize = meta.size();
                                    if tmp_status {
                                        log::info!("[{id_name}][{customer_id}]: FWD_FQ upload in progress - {filesize}b");
                                    } else {
                                        log::warn!("[{id_name}][{customer_id}]: Non-temporary FWD_FQ is uploading");
                                        log::info!("[{id_name}][{customer_id}]: FWD_FQ upload in progress - {filesize}b");
                                        // TODO: MD5
                                        // TODO: POST
                                        // TODO: GET all file to check 
                                    }
                                },
                                1 => {
                                    log::debug!("[{}][{}]: RVS_FQ is writing {:?}", id_name, customer_id, &path);
                                    let meta = fs::metadata(&path).unwrap();
                                    let filesize = meta.size();
                                    if tmp_status {
                                        log::info!("[{id_name}][{customer_id}]: RVS_FQ upload in progress - {filesize}b");
                                    } else {
                                        log::warn!("[{id_name}][{customer_id}]: Non-temporary FWD_FQ is uploading");
                                        log::info!("[{id_name}][{customer_id}]: RVS_FQ upload in progress - {filesize}b");
                                        // TODO: MD5
                                        // TODO: POST
                                        // TODO: GET all file to check 
                                    }
                                },
                                2 => {
                                    let meta = fs::metadata(&path).unwrap();
                                    let filesize = meta.size();
                                    log::warn!("[{}][{}]: FWD_MD5 is writing: {:?} - {filesize}b", id_name, customer_id, &path);
                                    log::debug!("[{}][{}]: FWD_MD5 is writing: {:?}", id_name, customer_id, &path);
                                    let (fwd, md5) = extract_md5(path, Regex::new(set[2]).unwrap());
                                    let mate: i8 = fwd.parse().unwrap();
                                    if mate != 1 { log::error!("{} not equal 1 when it should be 1", fwd); }
                                    // TODO: Warn if tmp as file is unexpectly large.
                                    log::warn!("[{id_name}][{customer_id}]: FWD_MD5 finished uploading - {md5}");
                                },
                                3 => {
                                    let meta = fs::metadata(&path).unwrap();
                                    let filesize = meta.size();
                                    log::warn!("[{}][{}]: RVS_MD5 is writing: {:?} - {filesize}b", id_name, customer_id, &path);
                                    log::debug!("[{}][{}]: RVS_MD5 is writing: {:?}", id_name, customer_id, &path);
                                    let (rvs, md5) = extract_md5(path, Regex::new(set[3]).unwrap());
                                    let mate: i8 = rvs.parse().unwrap();
                                    if mate != 2 { log::error!("{} not equal 2 when it should be 2", rvs); }
                                    // TODO: Warn if tmp as file is unexpectly large.
                                    log::warn!("[{id_name}][{customer_id}]: FWD_MD5 finished uploading - {md5}");
                                },
                                4 => {
                                    log::warn!("[{}][{}]: FWD_STAT is writing: {:?}", id_name, customer_id, &path);
                                    // TODO: Warn if tmp as file is unexpectly large.
                                    log::warn!("[{id_name}][{customer_id}]: FWD_STAT finished uploading");
                                },
                                5 => {
                                    log::warn!("[{}][{}]: RVS_STAT is writing: {:?}", id_name, customer_id, &path);
                                    // TODO: Warn if tmp as file is unexpectly large.
                                    log::warn!("[{id_name}][{customer_id}]: FWD_STAT finished uploading");
                                },
                                6 => {
                                    log::warn!("[{}][{}]: REPORT is writing: {:?}", id_name, customer_id, &path);
                                    // TODO: Warn if tmp as file is unexpectly large.
                                    log::warn!("[{id_name}][{customer_id}]: HTML_REPORT finished uploading");
                                },
                                _ => {}
                            }
                        }
                    } else {
                        log::error!("[{}]: File {:?} not matched with expected pattern! [(?<tmp>\\.?)(?<thaiomic_id>D[0-9]+)_(?<customer_id>[0-9]+)_(?<customer_index>[0-9]{{2}})-(?<flowcell_id>E[0-9]+)_(?<lane_id>L[0-9]{{2}})_(?<flowcell_index>[0-9]+)]", id_name, &path);
                    }
                } else {
                    //dir not match 
                    // TODO: Check if root folder
                    log::warn!("File is writing into watched directory directly!")
                    // TODO: Following the same criteria
                } 

            },
            EventKind::Remove(event) => {
                log::debug!("EventKind::Remove(Any): {:?}", &path);
                log::warn!("{:?} is removed", &path);
            },
            _ => {},
            
        }
             
    }


    pub fn extract_dir(path: &str, pattern: Regex) -> Option<(&str,&str,&str,&str,&str)>{
        let dir_name: Option<(&str,&str,&str,&str,&str)> = pattern.captures(path).map( |c| {
            let (_, [thaiomic_id, flowcell_id, year, month, day]) = c.extract();
            (thaiomic_id, flowcell_id, year, month, day)
        });
        dir_name
    }
    pub fn extract_fastq(path:&str , pattern: Regex) -> &str {
        let fq_mate: &str = pattern.captures(path).map(|c| {
            let (mate, [_]) = c.extract();
            mate
        }).unwrap();
        fq_mate
    }
    pub fn extract_fqstat(path:&str , pattern: Regex) -> &str {
        let stat_mate: &str = pattern.captures(path).map(|c| {
            let (mate, [_]) = c.extract();
            mate
        }).unwrap();
        stat_mate
    }

    pub fn extract_md5(path: &str , pattern: Regex) -> (&str, &str) {
        let md5: (&str,&str) = pattern.captures(path).map(|c| {
            let (_, [mate, suffix1, suffix2, md5]) = c.extract();
            (mate,md5)
        }).unwrap();
        md5
    }
    pub async fn checksum(file: PathBuf, md5: &str) -> (String, bool) {
        log::debug!("Computing MD5...");
        let mut file = fs::File::open(&file).unwrap(); 
        let mut hasher = Md5::new();
        let n = io::copy(&mut file, &mut hasher).unwrap();
        let hash = hasher.finalize();
        let computed_md5 = format!("{:x}", hash);
        if computed_md5 == md5 {
            log::info!("{computed_md5}, true");
            println!("{computed_md5}, true");
            (computed_md5, true)
        } else {
            println!("{computed_md5}, false");
            (computed_md5, false)
        }
    }


    
}





