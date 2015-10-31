
// YAML files as iptables configuration sources

extern crate clap;

use clap::{Arg, App};

fn main() {
    let matches = App::new("iptables-init")
        .version("1.0.0")
        .author("Samuel. <sam@infinitely.io>")
        .arg(Arg::with_name("CONFIG")
             .short("c")
             .long("config")
             .help("Yaml file as iptables configuration source")
             .takes_value(true))
        .get_matches();

    println!("YAML files as iptables configuration sources");

    let config = matches.value_of("CONFIG").unwrap_or("");
    println!("Value for config: {}", config);
}
