
// YAML files as iptables configuration sources

// Rust Core
use std::io::Read;
use std::fs::File;
use std::path::Path;

// Crate.io
extern crate clap;
extern crate yaml_rust;

use clap::{Arg, App};
use yaml_rust::{YamlLoader, YamlEmitter};

fn main() {

    let app = App::new("iptables-init")
        .version("1.0.0")
        .about("\nYAML files as iptables configuration sources")
        .arg(Arg::with_name("CONFIG")
             .multiple(true)
             .short("c")
             .long("config")
             .help("yaml file as iptables configuration source")
             .takes_value(true))
        .args_from_usage("-l --license 'Prints License'");

    let matches = app.get_matches();

    if let Some(ref f_paths) = matches.values_of("CONFIG") {
        for f_path in f_paths.iter() {
            generate_rules(f_path);
        }
    }

}

fn generate_rules(f_path: &str) {

    // Create a path for yaml configuration file
    let path = Path::new(&f_path);
    let display = path.display();

    // Open file
    let mut file = match File::open(&path) {
        Err(_)   => panic!("Failed to open file \"{}\"", display),
        Ok(file) => file,
    };

    // Create buffer string and read content
    let mut buffer_str = String::new();
    match file.read_to_string(&mut buffer_str) {
        Err(_) => panic!("Failed to read file \"{}\"", display),
        Ok(_)  => print!("{}", buffer_str),
    };

    let mut docs = match YamlLoader::load_from_str(&buffer_str) {
        Err(_)  => panic!("Failed to parse yaml file \"{}\"", display),
        Ok(doc) => doc,
    };

    // println!("file: {}", buffer_str);

}
