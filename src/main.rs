
// YAML files as iptables configuration sources

// Rust Core
use std::io::Read;
use std::fs::File;
use std::path::Path;
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
             .conflicts_with("license")
             .takes_value(true))
        .args_from_usage("-l --license 'Prints License'");

    let matches = app.get_matches();

    if let Some(ref f_paths) = matches.values_of("CONFIG") {
        for f_path in f_paths.iter() {
            read_yaml(f_path);
        }
    }

    if matches.is_present("license") {
        print_license();
    }

}

fn print_license() {
    let s:String = "
The MIT License (MIT)

Copyright (c) 2015 Samuel

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the \"Software\"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED \"AS IS\", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
".to_string();
    println!("{}", s);
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
    // Read all rules from yaml
    match doc {
        // Parse if template data is a hash object
        &Yaml::Hash(ref h) => {
            for (k, v) in h {
                match k.as_str().unwrap() {
                    // Parse `filter` rules
                    "filter" | "FILTER" => parse_filter(k, v),
                    // Parse custom section
                    _ => parse_section(k, v)
                }
            }
        },
        _ => {
            println!("Configuration template format is invalid");
            exit(1);
        }
    }
}

fn reset_rules() {
    let mut s:String = "# reset all rules".to_string();
    s.push_str("\niptables -F");
    s.push_str("\niptables -t nat -F");
    s.push_str("\niptables -t mangle -F");
    s.push_str("\niptables -X");
    println!("{}", s);
}

fn parse_section(id: &Yaml, doc: &Yaml) {
    match doc {
        &Yaml::Hash(ref h) => {
            println!("# {} rules", id.as_str().unwrap());
            for (k, v) in h {
                let k = k.as_str().unwrap();
                println!("{:?} - {:?}", k, v);
            }
        },
        _ => {
            println!("Configuration template format is invalid");
            exit(1);
        }
    }
}

fn parse_filter(id: &Yaml, doc: &Yaml) {
    // Check if filter section exists
    match doc {
        &Yaml::Hash(ref h) => {
            println!("# {} rules", id.as_str().unwrap());
            for (k, v) in h {
                let k = k.as_str().unwrap();
                let v = v.as_str().unwrap();
                match k {
                    "input" | "output" | "forward" | "INPUT" | "OUTPUT" | "FORWARD" => {
                        match v {
                            "drop" | "reject" | "accept" | "DROP" | "REJECT" | "ACCEPT" => println!("iptables -P {} {}", k, v),
                            _ => {
                                println!("Rules \"{}\" only accept options of \"drop\",\"reject\" or \"accept\"", k);
                                exit(1);
                            }
                        }
                    },
                    _ => println!("iptables -P {}", k)
                }
            }
        },
        _ => {
            println!("Configuration template format is invalid");
            exit(1);
        }
    }
}
