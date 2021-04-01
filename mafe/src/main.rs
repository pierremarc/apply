mod piet_cairo;
mod render;

use apply::{op::OpList, run_map};
use cairo::{Context, Format, ImageSurface, IoError};
use clap::{App, Arg, ArgMatches};
use parser::parse_str;
use piet::{
    kurbo::{Affine, Vec2},
    RenderContext,
};
use piet_cairo::CairoRenderContext;
use render::render;
use std::fs::read_to_string;
use std::fs::File;
use std::path::Path;

fn get_initial_transform(args: &Arguments) -> Affine {
    let x_scale = args.width() / args.extent_width();
    let y_scale = args.height() / args.extent_height();
    let init = Affine::IDENTITY;
    let translated = init
        * Affine::translate(Vec2 {
            x: -args.west() * x_scale,
            y: -args.south() * y_scale,
        });

    let scaled = translated * Affine::scale_non_uniform(x_scale, y_scale);
    dbg!(scaled)
}

fn render_png(args: Arguments, ops: &OpList) {
    let file_name = "ouput.png";
    let width: i32 = args.size[0] as i32;
    let height: i32 = args.size[1] as i32;
    let surface =
        ImageSurface::create(Format::ARgb32, width, height).expect("Can't create surface");
    let cairo_context = Context::new(&surface);
    let mut piet_context = CairoRenderContext::new(&cairo_context);
    piet_context.transform(get_initial_transform(&args));
    piet_context.save().unwrap();
    match render(&mut piet_context, ops) {
        Ok(_) => {
            File::create(file_name)
                .map_err(|e| IoError::Io(e))
                .and_then(|mut file| surface.write_to_png(&mut file))
                .unwrap();
        }
        Err(err) => {
            println!("Failed to render: {}", err);
        }
    }
}

fn run_main(args: Arguments) {
    let map_path = Path::new(args.mapfile.as_str());
    match read_to_string(&map_path) {
        Err(e) => println!("Failed to read {}: {}", map_path.display(), e),
        Ok(content) => {
            if let Ok(spec) = parse_str(&content) {
                // println!("<map\n {:?} \n/>", spec);
                if let Ok(ops) = run_map(spec) {
                    // for op in ops {
                    //     println!("op> {}", op);
                    // }
                    render_png(args, &ops);
                } else {
                    println!("run_map failed");
                }
            } else {
                println!("parse_str failed");
            }
        }
    }
}

#[derive(Debug)]
struct Arguments {
    extent: [f64; 4],
    size: [f64; 2],
    mapfile: String,
}

impl Arguments {
    fn from_matches(matches: ArgMatches) -> Result<Self, &str> {
        let mapfile = matches.value_of("map_file").ok_or("missing map file")?;
        let width = matches.value_of("width").ok_or("missing width")?;
        let height = matches.value_of("height").ok_or("missing height")?;
        let east = matches.value_of("east").ok_or("missing east")?;
        let north = matches.value_of("north").ok_or("missing north")?;
        let west = matches.value_of("west").ok_or("missing west")?;
        let south = matches.value_of("south").ok_or("missing south")?;

        let width: f64 = width.parse().map_err(|_| "failed to parse width")?;
        let height: f64 = height.parse().map_err(|_| "failed to parse height")?;
        let east: f64 = east.parse().map_err(|_| "failed to parse east")?;
        let north: f64 = north.parse().map_err(|_| "failed to parse north")?;
        let west: f64 = west.parse().map_err(|_| "failed to parse west")?;
        let south: f64 = south.parse().map_err(|_| "failed to parse south")?;

        Ok(dbg!(Arguments {
            extent: [west, south, east, north],
            size: [width, height],
            mapfile: String::from(mapfile),
        }))
    }

    fn width(&self) -> f64 {
        self.size[0]
    }

    fn height(&self) -> f64 {
        self.size[1]
    }
    fn extent_width(&self) -> f64 {
        self.extent[2] - self.extent[0]
    }

    fn extent_height(&self) -> f64 {
        self.extent[3] - self.extent[1]
    }
    fn south(&self) -> f64 {
        self.extent[1]
    }

    fn west(&self) -> f64 {
        self.extent[0]
    }
    fn north(&self) -> f64 {
        self.extent[3]
    }

    fn east(&self) -> f64 {
        self.extent[2]
    }
}

fn main() -> Result<(), &'static str> {
    let matches = App::new("Mafe")
        .version("0.1")
        .about("A convivial map processor")
        .arg(
            Arg::with_name("map_file")
                .short("f")
                .long("map_file")
                .value_name("MAP FILE")
                .help("The map file to process")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("width")
                .long("width")
                .help("width")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("height")
                .long("height")
                .help("height")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("east")
                .long("east")
                .help("east")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("north")
                .long("north")
                .help("north")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("west")
                .long("west")
                .help("west")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("south")
                .long("south")
                .help("south")
                .required(true)
                .takes_value(true),
        )
        .get_matches();

    let args = Arguments::from_matches(matches)?;
    run_main(args);
    Ok(())
}

#[cfg(test)]
mod test {
    use piet::kurbo;

    use super::*;
    // use crate::ast::*;
    #[test]
    fn initial_transform() {
        let args = Arguments {
            extent: [148284.9, 170598.2, 148957.2, 170993.6],
            size: [1000.0, 1000.0],
            mapfile: String::from("parser/data/map-format-geojson"),
        };

        let initial = get_initial_transform(&args);
        let origin = kurbo::Point::new(148284.9, 170598.2);
        let to = initial * origin;
        assert_eq!(to, kurbo::Point::new(0.0, 0.0));
    }
}
