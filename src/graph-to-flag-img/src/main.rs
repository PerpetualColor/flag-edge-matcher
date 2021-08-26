use std::fs::File;
use std::path::Path;
use std::collections::HashMap;
use std::collections::HashSet;
use std::io::BufReader;
use regex::Regex;

use shared::shared::*;
use image::{RgbaImage, DynamicImage};

const FLAG_DIMS: (u32, u32) = (320, 233);

fn read_multi_flags_from_file<P: AsRef<Path>>(path: P) -> HashMap<String, HashSet<String>> {
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);

    let output = serde_json::from_reader(reader).unwrap();

    output
}

fn read_graph_from_file<P: AsRef<Path>>(path: P) -> FlagGraph {
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);

    let serialized: FlagGraphSerialize = serde_json::from_reader(reader).unwrap();

    serialized.to_flag_graph()
}

fn load_flag<P: AsRef<Path>>(path: P) -> DynamicImage {
    image::open(path).unwrap().resize_exact(FLAG_DIMS.0, FLAG_DIMS.1, image::imageops::Gaussian)
}

fn main() {
    let available_files = std::fs::read_dir("./").unwrap();

    let search_regex = Regex::new(r"best_graph_found_(\d*)\.json").unwrap();
    let mut max = 0;
    for f in available_files {
        let path = f.unwrap().path();
        let file_name = path.file_name().unwrap().to_str().unwrap();

        if search_regex.is_match(file_name) {
            let groups: Vec<regex::Captures> = search_regex.captures_iter(file_name).collect();
            let group = &groups[0];

            let flags = (&group[1]).parse::<i32>().unwrap();
            if flags > max {
                max = flags;
            }
        }
    }

    let file_open = "./best_graph_found_".to_string() + &max.to_string() + ".json";
    println!("Opening file: {}", file_open);
    let arrangement = read_graph_from_file(file_open);
    let multi_flag_map = read_multi_flags_from_file("./multi_flags.json");

    let mut multi_flags: HashMap<String, Vec<String>> = HashMap::new();
    for (multiflag, flags) in multi_flag_map {
        multi_flags.insert(multiflag, flags.iter().map(|v| v.to_string()).collect());
    }

    let graph = arrangement.graph;
    let flag_count = graph.len();

    let mut min_x = 0;
    let mut max_x = 0;
    let mut min_y = 0;
    let mut max_y = 0;
    for (loc, _) in &graph {
        if loc.0 < min_x {
            min_x = loc.0;
        }
        if loc.0 > max_x {
            max_x = loc.0;
        }
        if loc.1 < min_y {
            min_y = loc.1;
        }
        if loc.1 > max_y {
            max_y = loc.1;
        }
    }
    let x_dim = (1 + max_x - min_x).abs() as u32;
    let y_dim = (1 + max_y - min_y).abs() as u32;

    let width = x_dim * FLAG_DIMS.0;
    let height = y_dim * FLAG_DIMS.1;
    
    println!("Creating image of {}x{} flags", x_dim, y_dim);
    let mut output_image = RgbaImage::new(width, height);

    let mut i = 1;

    for (loc, flag_id) in &graph {
        let x_graph_coord = loc.0 - min_x;
        let y_graph_coord = max_y - loc.1;

        let x_coord = x_graph_coord.abs() as u32 * FLAG_DIMS.0;
        let y_coord = y_graph_coord.abs() as u32 * FLAG_DIMS.1;

        let place_flag = multi_flags.get_mut(flag_id).unwrap().pop().unwrap();
        println!("({}/{}) Placing {} at {} {}", i, flag_count, place_flag, x_graph_coord, y_graph_coord);

        let flag_img = load_flag("./flags/".to_string() + &place_flag + ".png");
        let flag_view = flag_img.to_rgba8();

        image::imageops::overlay(&mut output_image, &flag_view, x_coord, y_coord);
        i += 1;
    }

    println!("Saving image...");
    output_image.save("output_image_".to_string() + &flag_count.to_string() + ".png").unwrap();
    println!("Done!");

}
