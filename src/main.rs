extern crate argparse;
extern crate chrono;
extern crate term_size;
extern crate unix_mode;
extern crate users;
extern crate rayon;

use argparse::{ArgumentParser, Store, StoreTrue};
use std::path::Path;
use std::process;
use path_absolutize::*;
use rayon::prelude::*;

mod utils;
use crate::utils::utils::*;

fn main() {
    let mut args = Arguments {
        path: String::from("."),
        show_version: false,
        show_long: false,
        show_all: false,
        show_size: false,
        show_dir_as_file: false,
        sort_field: String::from("name"),
    };
    {
        let mut parser = ArgumentParser::new();
        parser.set_description("Rewrite of GNU ls in rust");
        parser.refer(&mut args.path).add_argument(
            "path",
            Store,
            "path of the file or directiory to list",
        );
        parser.refer(&mut args.show_version).add_option(
            &["-v", "--version"],
            StoreTrue,
            "print the version information",
        );
        parser.refer(&mut args.show_long).add_option(
            &["-l", "--long"],
            StoreTrue,
            "provide detailed listing",
        );
        parser.refer(&mut args.show_all).add_option(
            &["-a", "--all"],
            StoreTrue,
            "include hidden files in the listing",
        );
        parser.refer(&mut args.show_size).add_option(
        	&["-s", "--show-size"], 
        	StoreTrue, 
        	"with -l, show sizes of files & directories (may be time consuming)"
        );
        parser.refer(&mut args.show_dir_as_file).add_option(
        	&["-d", "--show-dir-as-file"],
        	StoreTrue,
        	"show directory as a file, do not list the contents of it"	
        );
        parser.refer(&mut args.sort_field).add_option(
        	&["-S", "--sort"],
            Store,
            "with -l, sort the output according to the specified field, available fields are name (default), size & time"	
        );
        parser.parse_args_or_exit();
    }
    if args.show_version {
        println!("{}", VERSION);
        process::exit(0);
    }
    //println!("{:#?}", args);
    let terminal_width = term_size::dimensions().unwrap().0 - 10;

	let path = Path::new(&args.path).absolutize().unwrap();
	let abspath = String::from(path.to_str().unwrap());
	args.path = abspath.clone();

	let mut listing = get_listing(&args);
	if args.show_long {
		if args.sort_field.as_str() == "size" {
			listing.par_sort_by(|x,y| x.size.cmp(&y.size));
		} else if args.sort_field.as_str() == "time" {
			listing.par_sort_by(|x,y| x.mtime.cmp(&y.mtime));
		} else {
			listing.par_sort_by(|x,y| x.name.cmp(&y.name));
		}
	} else {
		listing.par_sort_by(|x,y| x.name.cmp(&y.name));
	}
	
	if args.show_long {
		let max_user_len = listing.par_iter().max_by_key(|x| x.username.chars().count())
							.unwrap().username.chars().count();

		for item in &listing {
			print!("{}{} ", C_RESET, item.perm_mask);
		    if args.show_size {
		    	let max_size_len = listing.par_iter().max_by_key(|x| x.humansize.chars().count())
		    						.unwrap().humansize.chars().count();
		        print!("{}{:>max_size_len$} ", C_GREEN, item.humansize);
		    }
		    print!("{}{:>max_user_len$} ", C_YELLOW, item.username);
		    print!("{}{:>12} ", C_THINBLUE, item.timestr);
		    println!("{}{}", item.color, item.name);
		}
	} else {
		let listnames: Vec<String> = listing
		.clone()
		.into_iter()
		.map(|item| item.name).collect();

		let listcolors: Vec<String> = listing
		.clone()
		.into_iter()
		.map(|item| item.color).collect();

		let mut c = 1;
		let mut chunks: Vec<Vec<String>>;
		let mut width_of_largest: Vec<usize> = Vec::new();
		loop {
		    chunks = get_chunks(&listnames, c);
		    let mut total = 0;
		    for chunk in &chunks {
		        total += largest_string(&chunk).chars().count();
			}
			//c += 1; println!("{:?}", total);
			if total < terminal_width {
		    	break;
			}
			c += 1;
		}
		for chunk in &chunks {
			let width = largest_string(&chunk).chars().count();
		    width_of_largest.push(width);
		}
	    let step = c;
		let cols = chunks.len();
		for j in 0..step {
		    c = j;
		    for i in 0..cols {
		    	if c < listnames.len() {
		            //let color = get_color(&listing[c]);
		            //print!("{:?} ", perm_mask);
		            print!("{}{:2$} ", listcolors[c], listnames[c], width_of_largest[i]);
		        }
		        c += step;
		    }
		    println!("{}", C_RESET);
		}
	}
    //println!("{:#?}", listing);
    //let mut pathlist =
    /*
    let listnames: Vec<String> = listing
    	.clone()
        .into_iter()
        .map(|item| {
        	let path = Path::new(&item);
            let mut name = path.to_str().unwrap();
            if path.file_name() != None {
            	name = path.file_name().unwrap().to_str().unwrap();
            }
            return String::from(name);
		}).collect();    
    if args.show_long {
		//println!("not implemented yet");
        for item in &listing {
        	let path = Path::new(&item);
            let color = get_color(&item);
            let name = path.file_name().unwrap().to_str().unwrap();
            let metadata = path.metadata().unwrap();
            let permissions = unix_mode::to_string(metadata.mode());
            let user = users::get_user_by_uid(metadata.uid()).unwrap();
            let user = user.name().to_str().unwrap();
            let mtime: DateTime<Local> = DateTime::from(Utc.timestamp(metadata.mtime(), 0));
            let mut timestr = mtime.format("%-d %b %H:%M");
            if Local::now().date().year() > mtime.date().year() {
            	timestr = mtime.format("%-d %b  %Y");
            }
            print!("{}{} ", C_RESET, permissions);
            if args.show_size {
                let humansize: String = get_humansize(get_size(&item));
                print!("{}{:>6} ", C_GREEN, humansize);
            }
            print!("{}{} ", C_YELLOW, user);
            print!("{}{:>12} ", C_THINBLUE, timestr);
            println!("{}{}", color, name);
        }
    } else {
        let mut c = 1;
        let mut chunks: Vec<Vec<String>>;
        let mut width_of_largest: Vec<usize> = Vec::new();
        loop {
            chunks = get_chunks(&listnames, c);
            let mut total = 0;
            for chunk in &chunks {
                total += largest_string(&chunk).chars().count();
            }
            //c += 1; println!("{:?}", total);
            if total < terminal_width {
                break;
            }
            c += 1;
		}
        for chunk in &chunks {
            let width = largest_string(&chunk).chars().count();
            width_of_largest.push(width);
        }
        let step = c;
        let cols = chunks.len();
        for j in 0..step {
            c = j;
            for i in 0..cols {
                if c < listnames.len() {
                    let color = get_color(&listing[c]);
                    //print!("{:?} ", perm_mask);
                    print!("{}{:2$} ", color, listnames[c], width_of_largest[i]);
                }
                c += step;
            }
            println!("{}", C_RESET);
        }
    }*/
}
