use std::fs;
use std::path::Path;
use jwalk::WalkDir;
use std::convert::AsRef;
use std::ffi::OsStr;
use rayon::prelude::*;
use crate::utils::fileinfo::FileInfo;

#[derive(Debug)]
pub struct Arguments {
    pub path: String,
    pub show_version: bool,
    pub show_long: bool,
    pub show_all: bool,
    pub show_size: bool,
    pub show_dir_as_file: bool,
    pub sort_field: String,
}

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const C_BLUE: &str = "\x1b[1;34m";
pub const C_THINBLUE: &str = "\x1b[0;34m";
pub const C_GREEN: &str = "\x1b[1;32m";
pub const C_CYAN: &str = "\x1b[1;36m";
pub const C_RESET: &str = "\x1b[0m";
pub const C_YELLOW: &str = "\x1b[1;33m";

pub fn get_size<T: AsRef<OsStr> + ?Sized>(pathstr: &T) -> u64 {
    let path = Path::new(pathstr);
    let metadata = &path.metadata().unwrap();
    if metadata.is_file() {
        let x: u64 = metadata.len();
        return x;
    } else if metadata.is_dir() {
        let mut total: u64 = 0;
        for entry in WalkDir::new(&path) {
            let metadata = entry.unwrap().metadata().unwrap();
            if metadata.is_file() {
                total += metadata.len();
            }
        }
        return total;
    } else {
        return 0;
    }
}

pub fn get_humansize(size: u64) -> String {
    let mut mutsize = size as f64;
    let mut size_unit = "";
    for unit in ["", "K", "M", "G"] {
        if mutsize < 1024.0 || unit == "G" {
            size_unit = unit;
            break;
        }
        mutsize /= 1024.0;
    }
    let humansize: String;
    if mutsize - mutsize.floor() > 0.0 {
        humansize = String::from(format!("{:.1}{}", mutsize, size_unit));
    } else {
        humansize = String::from(format!("{}{}", mutsize as i32, size_unit));
    }
    return humansize;
}

pub fn largest_string(list: &Vec<String>) -> &String {
    list.par_iter().max_by_key(|x| x.chars().count()).unwrap()
}

pub fn get_chunks(list: &Vec<String>, n: usize) -> Vec<Vec<String>> {
    list.chunks(n).map(|x| x.to_vec()).collect()
}

//pub fn type_of<T>(_: T) -> &'static str {
//	std::any::type_name::<T>()
//}

pub fn get_listing(args: &Arguments) -> Vec<FileInfo> {
	let arg_fileinfo = FileInfo::new(&args.path, false, false);
	let mut listing = Vec::new();
	if arg_fileinfo.filetype == 1 {
		if args.show_dir_as_file {
			listing.push(FileInfo::new(&args.path, args.show_long, args.show_size));
		} else {
	        for file in fs::read_dir(&arg_fileinfo.pathstr).unwrap() {
	            let pathbuf = file.as_ref().unwrap().path();
	            let pathstr = pathbuf.to_str().unwrap();
	            if file.unwrap().file_name().to_str().unwrap().chars().nth(0) == Some('.') {
	                if args.show_all {
	                    listing.push(FileInfo::new(
	                    	&String::from(pathstr), 
	                    	args.show_long, 
	                    	args.show_size
	                    ));
	                }
	            } else {
	                listing.push(FileInfo::new(
	               		&String::from(pathstr), 
	                	args.show_long, 
	                	args.show_size
	                ));
	            }
	        }
	        //listing.sort();
	    }
	} else {
		listing.push(FileInfo::new(&args.path, args.show_long, args.show_size));
	}
	listing
}
