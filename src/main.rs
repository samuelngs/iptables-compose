
// YAML files as iptables configuration sources

// Rust Core
use std::io::Read;
use std::fs::File;
use std::path::Path;
use std::ascii::AsciiExt;
use std::process::exit;

// Crate.io
extern crate clap;
extern crate yaml_rust;

use clap::{Arg, App};
use yaml_rust::{Yaml, YamlLoader};

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
            read_yaml(f_path);
        }
    }

}

fn read_yaml(f_path: &str) {

    // Create a path for yaml configuration file
    let path = Path::new(&f_path);
    let display = path.display();

    // Open file
    let mut file = match File::open(&path) {
        Err(_)   => {
            println!("Failed to open file \"{}\"", display);
            exit(1);
        },
        Ok(file) => file,
    };

    // Create buffer string and read content
    let mut buffer_str = String::new();
    match file.read_to_string(&mut buffer_str) {
        Err(_)  => {
            println!("Failed to read file \"{}\"", display);
            exit(1);
        },
        Ok(str) => str,
    };

    // Parse yaml file
    let yaml = match YamlLoader::load_from_str(&buffer_str) {
        Err(_)  => {
            println!("Failed to parse yaml data: \"{}\"", display);
            exit(1);
        },
        Ok(yaml) => yaml,
    };

    // Check if yaml file has no data, exits program if no data
    if yaml.is_empty() {
        println!("\"{}\" has no configuration(s)", display);
        exit(1);
    };

    // Validate configuration template format
    match yaml[0].as_hash() {
        Some(doc) => doc,
        None => {
            println!("Configuration template format is invalid: {}", display);
            exit(1);
        }
    };

    // Parse configuration from YAML configuration file
    parse_yaml(&yaml[0]);

}

fn parse_yaml(doc: &Yaml) {
    // Reset all rules
    reset_rules();
    // Parse `filter` rules
    parse_filter(&doc);
}

fn reset_rules() {
    let mut s:String = "iptables -F".to_string();
    s.push_str("\niptables -t nat -F");
    s.push_str("\niptables -t mangle -F");
    s.push_str("\niptables -X");
    println!("{}", s);
}

fn parse_filter(doc: &Yaml) {
    // Set target name
    let target = "filter";
    // Check if filter section exists
    match doc[target].is_badvalue() {
        false => {
            let labels = ["input", "forward", "output"];
            for label in labels.iter() {
                match_filter(doc, target, label);
            }
        },
        _ => {}
    }
}

fn match_filter(doc: &Yaml, target: &str, label: &str) {
    let rule = doc[target][label].as_str();
    match rule {
        Some("DROP") |
        Some("ACCEPT") |
        Some("REJECT") => println!("iptables -P {} {}", label.to_ascii_uppercase(), rule.unwrap()),
        _ => println!("iptables -P {} ACCEPT", label.to_ascii_uppercase())
    }
}

