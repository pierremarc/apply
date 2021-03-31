mod piet_cairo;
mod render;

use apply::{op::OpList, run_map};
use cairo::{Context, Format, ImageSurface};
use clap::{App, Arg};
use parser::parse_str;
use piet_cairo::CairoRenderContext;
use render::render;
use std::fs::read_to_string;
use std::fs::File;
use std::path::Path;

fn render_png(ops: &OpList) {
    let file_name = "ouput.png";
    let surface = ImageSurface::create(Format::ARgb32, 1024, 1024).expect("Can't create surface");
    let cairo_context = Context::new(&surface);
    let mut piet_context = CairoRenderContext::new(&cairo_context);
    if let Ok(()) = render(&mut piet_context, ops) {
        File::create(path)
            .and_then(|file| surface.write_to_png(&mut file).map_err(Into::into))
            .unwrap();
    } else {
        println!("Failed to render");
    }
}

fn run_main(map_path: String) {
    let map_path = Path::new(map_path.as_str());
    match read_to_string(&map_path) {
        Err(e) => println!("Failed to read {}: {}", map_path.display(), e),
        Ok(content) => {
            if let Ok(spec) = parse_str(&content) {
                // println!("<map\n {:?} \n/>", spec);
                if let Ok(ops) = run_map(spec) {
                    // for op in ops {
                    //     println!("op> {}", op);
                    // }
                    render_png(&ops);
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

    let map_path = matches.value_of("map_file").unwrap();
    run_main(String::from(map_path));
}
