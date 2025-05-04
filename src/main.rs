use std::env;
use std::io::prelude::*;
use std::fs;
use std::path;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct PawConfig {
    header: String,
    footer: String,
    css_path: String,
    js_path: String
}

struct BarkFile {
    title: String,
    description: String,
    content: String, // markdown
}

fn read_bark_variables(bark_content: &String) -> Result<BarkFile, &'static str> {
    let mut bark_file_vars: BarkFile = BarkFile {
        title: String::from(""),
        description: String::from(""),
        content: String::from(""),
    }; 

    let split_content: Vec<&str> = bark_content.split("!content").collect();

    if split_content.len() < 2 {
        return Err("Invalid bark file :3");
    }

    let metadata = match split_content[0].split_once("\n") {
        Some(metadata) => metadata,
        None => return Err("Invalid metadata structure"),
    };

    let title = metadata.0;
    let description = metadata.1;
    let content = split_content[1];
    bark_file_vars.title = title.to_owned();
    bark_file_vars.description = description.to_owned();
    bark_file_vars.content = content.to_owned();
    return Ok(bark_file_vars);
}

fn process_bark(bark_path: &path::Path, config: &PawConfig) -> () {
	let parent_path = match bark_path.parent() {
		Some(parent_path) => parent_path,
		None => panic!("Couldn't extract parent path!"),
	};

	let contents = fs::read_to_string(bark_path).expect("Couldn't convert to a string!");
    let variables = match read_bark_variables(&contents) {
        Ok(variables) => variables,
        Err(e) => {
            println!("{e}! Skipping file.");
            return;
        }
    };
    println!("{:?}", markdown::to_html(variables.content.as_str()));
	let regexp = Regex::new(r"!paw.*!").expect("Error while creating a regexp (for some reason?)");
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

	file.write_all(markdown::to_html(&output).as_bytes()).expect("Unable to write to a file");
}

fn process_markdown(md_path: &path::Path) -> String {
	let contents = fs::read_to_string(md_path).expect("Unable to convert markdown to HTML!");
	return markdown::to_html(&contents.to_owned());
}

fn process_file(file_path: &path::Path, config: &PawConfig) -> () {	
	let extension = file_path.extension();

	match extension {
		Some(extension) => {
			if extension == "bark" {
				process_bark(file_path, config);
			}
		},
		None => (),
	};
}

fn process_directory(dir_path: &path::Path, config: &PawConfig) -> () {
	let directory = fs::read_dir(dir_path).expect("Unable to read directory");

	for entry in directory {
		match entry {
			Ok(entry) => {
				let file_type = entry.file_type();

				if file_type.unwrap().is_dir() {
					process_directory(&entry.path(), config);
				} else {
					process_file(&entry.path(), config);
				}
			},
			Err(e) => panic!("{e}"),
		};
	}
}

fn read_config_file(dir_path: &path::Path) -> Result<PawConfig, &'static str> {
	// It's only mutable because "any" method requires a mutable reference to self :3
	let mut directory = fs::read_dir(dir_path).expect("Unable to read directory");

	if directory.any(|entry| {
		match entry {
			Ok(entry) => return entry.file_name() == "paw.json",
			Err(_) => return false,
		};
	}) {
        
        let json_path = path::Path::new(dir_path);
	    let contents = fs::read_to_string(json_path.join("paw.json")).expect("Couldn't convert to a string!");
        match serde_json::from_str::<PawConfig>(contents.as_str()) {
            Ok(cfg) => {
                let mut config: PawConfig = PawConfig{
                    header: String::from(""),
                    footer: String::from(""),
                    css_path: String::from(""),
                    js_path: String::from(""),
                };
                config.header = path::Path::new(dir_path).join(cfg.header).into_os_string().into_string().unwrap();
                config.footer = path::Path::new(dir_path).join(cfg.footer).into_os_string().into_string().unwrap();
                config.css_path = path::Path::new(dir_path).join(cfg.css_path).into_os_string().into_string().unwrap();
                config.js_path = path::Path::new(dir_path).join(cfg.js_path).into_os_string().into_string().unwrap();
                return Ok(config);
            },
            Err(_) => return Err("AAAAAAAAAAAAAAAAAA"), 
        };
	}

    return Err("AAAAAAAAAAAa2");
}

fn main() {
	let current_working_directory = env::current_dir().expect("Unable to fetch current working directory");
	let cwd_path = current_working_directory.as_path();
    let testing_path = path::Path::new("testing");

	let cfg = match read_config_file(testing_path) {
        Ok(cfg) => cfg,
        Err(_) => panic!("bwe"),
    }; 
	process_directory(testing_path, &cfg);
}

