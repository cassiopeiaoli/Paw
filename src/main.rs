use std::env;
use std::io::prelude::*;
use std::fs;
use std::path;
use serde::{Deserialize, Serialize};
use colored::Colorize;
use pathdiff::diff_paths;

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

fn adjust_relative_css_js_path(bark_path: &path::Path, config: &PawConfig) -> (String, String) {
    let string_path = bark_path.to_str().expect("nuh uh");
    let matches: Vec<&str> = string_path.matches("/").collect();

    if matches.len() <= 1 {
        return (String::from(config.css_path.clone()), String::from(config.js_path.clone()));
    }

    // diff_paths does a weird thing where it will always add ../ if files are in the same path???
    let css_path = diff_paths(&config.css_path, bark_path).unwrap().to_str().unwrap().to_owned().replacen("../", "", 1); 
    let js_path = diff_paths(&config.js_path, bark_path).unwrap().to_str().unwrap().to_owned().replacen("../", "", 1); 

    return (css_path, js_path);
}

fn edit_metadata(bark_path: &path::Path, config: &PawConfig, variables: &BarkFile) -> String {
    let title = String::from("<title>") + &variables.title + "</title>";
    let description = String::from("<meta name='description' content='") + &variables.description + "'>";
    let css_js_links = adjust_relative_css_js_path(bark_path, config);
    let css = String::from("<link rel='stylesheet' href='",) + css_js_links.0.as_str() + "'>";
    let js = String::from("<script defer src='") + css_js_links.1.as_str() + "'></script>";
    return config.header
        .replacen("!title", title.as_str(), 1)
        .replacen("!description", description.as_str(), 1)
        .replacen("!css", css.as_str(), 1)
        .replacen("!js", js.as_str(), 1);
}

fn process_bark(bark_path: &path::Path, config: &PawConfig) -> () {
    println!("{}", config.css_path);
    println!("Processing: {}", bark_path.to_str().unwrap().yellow());
	let parent_path = match bark_path.parent() {
		Some(parent_path) => parent_path,
		None => panic!("Couldn't extract parent path!"),
	};

	let contents = fs::read_to_string(bark_path).expect("Couldn't convert to a string!");
    let variables = match read_bark_variables(&contents) {
        Ok(variables) => variables,
        Err(e) => {
            eprintln!("Processing {} not successful", bark_path.to_str().unwrap().red());
            eprintln!("{}! Skipping file.", e);
            return;
        }
    };
    let header_content = edit_metadata(bark_path, config, &variables);
    let parsed_content = markdown::to_html(variables.content.as_str());
    let output_content = String::from("<!doctype html><html>") + &header_content + "<body>" + &parsed_content + &config.footer + "</body></html>";

	let output_filename = String::from(bark_path.file_stem().unwrap().to_str().unwrap()) + ".html";
	let output_path = parent_path.join(output_filename);
	let mut file = fs::File::create(&output_path).expect("Couldn't create file");

	file.write_all(output_content.as_bytes()).expect("Unable to write to a file");
    println!("Processing {} {}\n", bark_path.to_str().unwrap().green(), "successful".green());
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
	    let contents = fs::read_to_string(json_path.join("paw.json")).expect("Couldn't read paw.json");
        match serde_json::from_str::<PawConfig>(contents.as_str()) {
            Ok(cfg) => {
                let mut config: PawConfig = PawConfig{
                    header: String::from(""),
                    footer: String::from(""),
                    css_path: String::from(""),
                    js_path: String::from(""),
                };
                let header_path = path::Path::new(dir_path).join(cfg.header).into_os_string().into_string().unwrap();
                let footer_path =path::Path::new(dir_path).join(cfg.footer).into_os_string().into_string().unwrap();
                config.header = match fs::read_to_string(header_path) {
                    Ok(content) => content,
                    Err(_) => return Err("Unable to find/read header file"),
                };
                config.footer = match fs::read_to_string(footer_path) {
                    Ok(content) => content,
                    Err(_) => return Err("Unable to find/read footer file"),
                };
                config.css_path = path::Path::new(dir_path).join(cfg.css_path).into_os_string().into_string().unwrap();
                config.js_path = path::Path::new(dir_path).join(cfg.js_path).into_os_string().into_string().unwrap();
                return Ok(config);
            },
            Err(_) => return Err("Incorrect structure of paw.json"), 
        };
	}

    return Err("paw.json not found");
}

fn main() {
    println!("{}", "PAW!".white().bold().on_purple());
    println!("{} by {}\n", "A Static Site Generator".white().on_purple(), "itchydog".bold().italic().white().on_purple());
	let current_working_directory = env::current_dir().expect("Unable to fetch current working directory");
	let cwd_path = current_working_directory.as_path();
    //let testing_path = path::Path::new("testing");
    println!("{:?}", cwd_path);

	let cfg = match read_config_file(cwd_path) {
        Ok(cfg) => cfg,
        Err(e) => panic!("{}", e),
    };

    process_directory(cwd_path, &cfg);
}

