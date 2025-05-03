use std::env;
use std::io::prelude::*;
use std::fs;
use std::path;
use regex::Regex;

fn process_bark(bark_path: &path::PathBuf) -> () {
	let parent_path = match bark_path.parent() {
		Some(parent_path) => parent_path,
		None => panic!("Couldn't extract parent path!"),
	};
	let contents = fs::read_to_string(bark_path).expect("Couldn't convert to a string!");

	let regexp = match Regex::new(r"!paw.*!") {
		Ok(regexp) => regexp,
		Err(e) => panic!("Error while creating a regexp (for some reason?). Reason: {e}"),
	};
	let string = contents.to_owned();
	let mut output = contents.clone();

	for capture in regexp.captures_iter(&string) {
		let captured_str = match capture.get(0) {
			Some(captured_str) => captured_str,
			None => break,
		}.as_str();
		let md_filename = captured_str.replace("!paw", "").replace("!", "");
		let md_path = parent_path.join(md_filename.trim());
		
		if md_path.exists() {
			let converted_md = process_markdown(&md_path);
			output = output.replace(captured_str, converted_md.as_str());
		} else {
			output = output.replace(captured_str, "");
		}
	}

	let output_filename = String::from(bark_path.file_stem().unwrap().to_str().unwrap()) + ".html";
	let output_path = parent_path.join(output_filename);
	
	let mut file = match fs::File::create(&output_path) {
		Err(e) => panic!("Couldn't create a file. Reason: {e}"),
		Ok(file) => file,
	};

	match file.write_all(output.as_bytes()) {
		Err(e) => panic!("Couldn't write to the file. Reason: {e}"),
		Ok(_) => println!("Finished processing {:?}", bark_path),
	};
}

fn process_markdown(md_path: &path::PathBuf) -> String {
	let contents = fs::read_to_string(md_path).expect("Unable to convert markdown to HTML!");
	return markdown::to_html(&contents.to_owned());
}

fn process_file(file_path: &path::PathBuf) -> () {	
	let extension = file_path.extension();
	match extension {
		Some(extension) => {
			if extension == "bark" {
				process_bark(file_path);
			}
		},
		None => (),
	};
}

fn process_directory(dir_path: &path::PathBuf) -> () {
	let directory = match fs::read_dir(dir_path) {
		Ok(directory) => directory,
		Err(e) => panic!("Unabled to read directory, reason: {e}"),
	};

	for entry in directory {
		match entry {
			Ok(entry) => {
				let file_type = entry.file_type();
				if file_type.unwrap().is_dir() {
					process_directory(&entry.path());
				} else {
					process_file(&entry.path());
				}
			},
			Err(e) => panic!("{e}"),
		};
	}
}

fn validate_paw_project(dir_path: &path::PathBuf) -> () {
	// It's only mutable because "any" method requires a mutable referencce to self :3
	let mut directory = match fs::read_dir(dir_path) {
		Ok(directory) => directory,
		Err(e) => panic!("Unabled to read directory, reason: {e}"),
	};

	if !directory.any(|entry| {
		match entry {
			Ok(entry) => return entry.file_name() == "paw",
			Err(e) => return false,
		};
	}) {
		panic!("This directory is not a correct Paw! directory -w-");
	}
}

fn main() {
	let current_working_directory = match env::current_dir() {
		Ok(current_working_directory) => current_working_directory,
		Err(e) => panic!("Unable to fetch current working directory, reason: {e}"),
	};
	
	validate_paw_project(&current_working_directory);
	process_directory(&current_working_directory);
}

