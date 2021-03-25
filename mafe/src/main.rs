use apply::run_map;
use clap::{App, Arg};
use parser::parse_str;
use std::fs::read_to_string;
use std::path::Path;

fn run_main(map_path: &Path) {
    match read_to_string(map_path) {
        Err(e) => println!("Failed to read {}: {}", map_path.display(), e),
        Ok(content) => {
            if let Ok(spec) = parse_str(&content) {
                // println!("<map\n {:?} \n/>", spec);
                if let Ok(ops) = run_map(spec) {
                    for op in ops {
                        println!("op> {}", op);
                    }
                } else {
                    println!("run_map failed");
                }
            } else {
                println!("parse_str failed");
            }
        }
    }
}

fn main() {
    let map_file = Arg::with_name("map_file")
        .short("f")
        .long("map_file")
        .value_name("MAP FILE")
        .help("The map file to process")
        .required(true)
        .takes_value(true);

    let matches = App::new("Mafe")
        .version("0.1")
        .about("A convivial map processor")
        .arg(map_file)
        .get_matches();

    let map_path = Path::new(matches.value_of("map_file").unwrap());

    run_main(map_path)
}
