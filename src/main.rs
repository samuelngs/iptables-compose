
// YAML files as iptables configuration sources

// Rust Core
use std::ascii::AsciiExt;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process::exit;

// Crate.io
extern crate clap;
extern crate yaml_rust;

use clap::{Arg, App};
use yaml_rust::{Yaml, YamlLoader};

fn main() {

    let app = App::new("iptables-compose")
        .version("1.1.0")
        .global_version(true)
        .unified_help_message(true)
        .arg_required_else_help(true)
        .about("\nYAML files as iptables configuration sources")
        .arg(Arg::with_name("CONFIG")
             .multiple(false)
             .help("yaml file as iptables configuration source")
             .conflicts_with("license")
             .index(1))
        .arg(Arg::with_name("RESET")
             .short("r")
             .long("reset")
             .help("reset iptables rules\n")
             .requires("CONFIG"))
        .args_from_usage("-l --license 'Prints License'");

    let matches = app.get_matches();

    if let Some(ref f_path) = matches.value_of("CONFIG") {
        // Reset rules if "reset" argument is present
        if matches.is_present("RESET") {
            reset_rules();
        }
        read_yaml(f_path);
    }

    if matches.is_present("license") {
        print_license();
    }

}

fn print_license() {
    let s:String = "
The MIT License (MIT)

Copyright (c) 2015 Samuel <sam@infinitely.io>

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
    // Read all rules from yaml
    match doc {
        // Parse if template data is a hash object
        &Yaml::Hash(ref h) => {
            for (k, v) in h {
                match k.as_str().unwrap() {
                    // Parse `filter` rules
                    "filter" | "FILTER" => parse_filter(v),
                    // Parse custom section
                    _ => parse_section(v)
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
    let mut s:String = "iptables -F".to_string();
    s.push_str("\niptables -X");
    s.push_str("\niptables -t nat -F");
    s.push_str("\niptables -t nat -X");
    s.push_str("\niptables -t mangle -F");
    s.push_str("\niptables -t mangle -X");
    println!("{}", s);
}

fn parse_section(doc: &Yaml) {
    match doc {
        &Yaml::Hash(ref h) => {
            for (k, v) in h {
                let k = k.as_str().unwrap();
                match k {
                    "ports" | "PORTS" => parse_ports(v),
                    _ => {
                        println!("Rules \"{}\" is not available", k);
                        exit(1);
                    }
                }
            }
        },
        _ => {
            println!("Configuration template format is invalid");
            exit(1);
        }
    }
}

fn parse_ports(doc: &Yaml) {
    match doc {
        &Yaml::Array(ref v) => {
            for x in v {
                parse_port_item(x);
            }
        },
        _ => {
            println!("Configuration \"ports\" format is invalid");
            exit(1);
        }
    }
}

fn parse_port_item(doc: &Yaml) {
    match doc {
        &Yaml::Hash(_) => {
            if doc["port"].is_badvalue() {
                println!("Port is not defined");
            }
            let port = doc["port"].as_i64();
            match port {
                Some(port) if port > -1 => (),
                _ => {
                    if port.unwrap() <= -1 {
                        println!("Port has to be greater or equals to 0");
                    } else {
                        println!("Port is not invalid");
                    }
                    exit(1);
                }
            }
            let mut cmd:String = "iptables".to_string();
            if ! doc["forward"].is_badvalue() {
                cmd.push_str(" -t nat -A PREROUTING");
            } else {
                let direction = doc["type"].as_str().unwrap_or("input");
                match direction {
                    "input" | "output" | "forward" | "INPUT" | "OUTPUT" | "FORWARD" => {
                        cmd.push_str(" -I ");
                        cmd.push_str(&direction.to_ascii_uppercase());
                    },
                    _ => {
                        println!("Direction value is invalid");
                        exit(1);
                    }
                }
            }
            match doc["subnet"] {
                Yaml::Array(ref v) => {
                    cmd.push_str(" -s ");
                    let mut y: i32 = 0;
                    for x in v {
                        if y > 0 {
                            cmd.push_str(",");
                        }
                        cmd.push_str(x.as_str().unwrap_or("0.0.0.0/0"));
                        y += 1;
                    }
                },
                _ => {}
            }
            let protocol = doc["protocol"].as_str().unwrap_or("tcp");
            match protocol {
                "tcp" | "udp" | "TCP" | "UDP" => {
                    cmd.push_str(" -p ");
                    cmd.push_str(protocol);
                    cmd.push_str(" -m ");
                    cmd.push_str(protocol);
                },
                _ => {
                    println!("Protocol value is invalid");
                    exit(1);
                }
            }
            cmd.push_str(" --dport ");
            cmd.push_str(&port.unwrap().to_string());
            if ! doc["forward"].is_badvalue() {
                let forward = doc["forward"].as_i64();
                match forward {
                    Some(forward) if forward > -1 => (),
                    _ => {
                        if forward.unwrap() <= -1 {
                            println!("forward port has to be greater or equals to 0");
                        } else {
                            println!("forward port is not invalid");
                        }
                        exit(1);
                    }
                }
                cmd.push_str(" -j REDIRECT --to-port ");
                cmd.push_str(&forward.unwrap().to_string());
            } else {
                let allow = doc["allow"].as_bool().unwrap_or(true);
                cmd.push_str(" -j ");
                match allow {
                    true => {
                        cmd.push_str("ACCEPT");
                    },
                    _ => {
                        cmd.push_str("DROP");
                    }
                }
            }
            println!("{}", cmd);
        },
        _ => {
            println!("Configuration \"ports\" item format is invalid");
            exit(1);
        }
    }
}

fn parse_filter(doc: &Yaml) {
    // Check if filter section exists
    match doc {
        &Yaml::Hash(ref h) => {
            for (k, v) in h {
                let k = k.as_str().unwrap();
                let v = v.as_str().unwrap();
                match k {
                    "input" | "output" | "forward" | "INPUT" | "OUTPUT" | "FORWARD" => {
                        match v {
                            "drop" | "reject" | "accept" | "DROP" | "REJECT" | "ACCEPT" => println!("iptables -P {} {}", k.to_ascii_uppercase(), v.to_ascii_uppercase()),
                            _ => {
                                println!("Rules \"{}\" only accept options of \"drop\",\"reject\" or \"accept\"", k);
                                exit(1);
                            }
                        }
                    },
                    _ => println!("iptables -P {}", k.to_ascii_uppercase())
                }
            }
        },
        _ => {
            println!("Configuration template format is invalid");
            exit(1);
        }
    }
}
