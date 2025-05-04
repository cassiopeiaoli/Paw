use std::env;
use std::io::prelude::*;
use std::fs;
use std::path;
use regex::Regex;

fn process_bark(bark_path: &path::Path) -> () {
	let parent_path = match bark_path.parent() {
		Some(parent_path) => parent_path,
		None => panic!("Couldn't extract parent path!"),
	};

	let contents = fs::read_to_string(bark_path).expect("Couldn't convert to a string!");
	let regexp = Regex::new(r"!paw.*!").expect("Error while creating a regexp (for some reason?). Reason: {e}");
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
	let mut file = fs::File::create(&output_path).expect("Couldn't create file");

	// We also allow markdown in .bark files to make website-making easier.
	// We shooooooould probably have some marked spots where we don't want MD to be processed
	// But that's a problem for future Cassiopeia
	file.write_all(markdown::to_html(&output).as_bytes()).expect("Unable to write to a file");
}

fn process_markdown(md_path: &path::Path) -> String {
	let contents = fs::read_to_string(md_path).expect("Unable to convert markdown to HTML!");
	return markdown::to_html(&contents.to_owned());
}

fn process_file(file_path: &path::Path) -> () {	
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

fn process_directory(dir_path: &path::Path) -> () {
	let directory = fs::read_dir(dir_path).expect("Unable to read directory");

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

fn validate_paw_project(dir_path: &path::Path) -> () {
	// It's only mutable because "any" method requires a mutable reference to self :3
	let mut directory = fs::read_dir(dir_path).expect("Unable to read directory");

	if !directory.any(|entry| {
		match entry {
			Ok(entry) => return entry.file_name() == "paw",
			Err(_) => return false,
		};
	}) {
		panic!("This directory is not a correct Paw! directory -w-");
	}
}

fn main() {
	let current_working_directory = env::current_dir().expect("Unable to fetch current working directory");
	let cwd_path = current_working_directory.as_path();

	validate_paw_project(cwd_path);
	process_directory(cwd_path);
}

