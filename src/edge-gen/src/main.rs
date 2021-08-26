use image::io::Reader as ImageReader;
use image::Rgb;
use shared::shared::*;

use std::fs::File;
use std::io::prelude::*;

use serde::{Serialize, Deserialize};

type Pixel = Rgb<u8>;

const COLORS: [(&str, Pixel); 9] = [ 
    ("Red", Rgb([255, 0, 0])),
    ("Green", Rgb([0, 255, 0])),
    ("Blue", Rgb([0, 0, 255])),
    ("Yellow", Rgb([255, 255, 0])),
    ("Cyan", Rgb([0, 255, 255])),
    ("Magenta", Rgb([255, 0, 255])),
    ("Orange", Rgb([255, 128, 0])),
    ("White", Rgb([255, 255, 255])),
    ("Black", Rgb([0, 0, 0])),
];

const PROPORTION_DENOM: u32 = 24;

const BLACK_THRESH: u8 = 75;

struct SideIterator<'a> {
    side: Sides,
    idx: u32,
    img: &'a image::ImageBuffer<image::Rgb<u8>, std::vec::Vec<u8>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct FlagEdges {
    id: String,
    top: EdgeInfo,
    right: EdgeInfo,
    bottom: EdgeInfo,
    left: EdgeInfo,
}

impl SideIterator<'_> {
    pub fn new(side: Sides, img: &image::ImageBuffer<Pixel, Vec<u8>>) -> SideIterator {
        SideIterator {
            side,
            idx: 0,
            img,
        }
    }

    pub fn length(&self) -> u32 {
        match self.side {
            Sides::TOP | Sides::BOTTOM => self.img.width(),
            Sides::LEFT | Sides::RIGHT => self.img.height(),
        }
    }
}

impl Iterator for SideIterator<'_> {
    type Item = Pixel;

    fn next(&mut self) -> Option<Self::Item> {
        let idx = self.idx;
        self.idx += 1;
        
        match self.side {
            Sides::TOP => {
                if idx < self.img.width() {
                    Some(*self.img.get_pixel(idx, 0))
                } else {
                    None
                }
            },
            Sides::RIGHT => {
                if idx < self.img.height() {
                    Some(*self.img.get_pixel(self.img.width() - 1, idx))
                } else {
                    None
                }
            },
            Sides::BOTTOM => {
                if idx < self.img.width() {
                    Some(*self.img.get_pixel(idx, self.img.height() - 1))
                } else {
                    None
                }
            },
            Sides::LEFT => {
                if idx < self.img.height() {
                    Some(*self.img.get_pixel(0, idx))
                } else {
                    None
                }
            }
        }
    }
}

fn nearest_color(color: &Pixel) -> String {
    let mut best = (COLORS[0], -1.0);
    let not_black = color[0] > BLACK_THRESH || color[1] > BLACK_THRESH || color[2] > BLACK_THRESH;
    for c in &COLORS {
        let mut d_sq = 0.0;
        for i in 0..3 {
            d_sq += ((color[i] as f32) - (c.1[i] as f32)).powi(2);
        }
        if c.0 == "Black" {
            if (best.1 == -1.0 || d_sq < best.1) && !not_black {
                best = (*c, d_sq);
            }
        } else if best.1 == -1.0 || d_sq < best.1 {
            best = (*c, d_sq);
        }
    }
    String::from(best.0.0)
}

fn build_side_info(iter: &mut SideIterator) -> EdgeInfo {
    let mut output = Vec::new();
    let mut current_count = 1;
    let nx = iter.next().unwrap();
    let mut current_color = nearest_color(&nx);
    let iter_length = iter.length();

    let to_prop = |count: u32| {
        ((count * PROPORTION_DENOM) as f32 / (iter_length as f32)).round() as u32
    };

    let push_value = |current_color: String, current_count: u32, output: &mut Vec<(String, u32)>| {
        let prop = to_prop(current_count);
        if prop > 0 {
            output.push((current_color, prop));
        }
    };

    for px in iter {
        let col = nearest_color(&px);
        if col == current_color {
            current_count += 1;
        } else {
            push_value(current_color, current_count, &mut output);
            current_count = 1;
            current_color = col;
        }
    }
    push_value(current_color, current_count, &mut output);
    output
}

fn build_flag_info(flag: std::path::PathBuf) -> FlagEdges {
    let id = flag.file_stem().unwrap().to_str().unwrap().to_string();
    println!("id: {}", id);
    let img = ImageReader::open(flag).unwrap().decode().unwrap().to_rgb8();

    FlagEdges {
        id,
        top: build_side_info(&mut SideIterator::new(Sides::TOP, &img)),
        right: build_side_info(&mut SideIterator::new(Sides::RIGHT, &img)),
        bottom: build_side_info(&mut SideIterator::new(Sides::BOTTOM, &img)),
        left: build_side_info(&mut SideIterator::new(Sides::LEFT, &img)),
    }
}

fn main() {
    let mut output_data: Vec<FlagEdges> = Vec::new();

    let flag_files = std::fs::read_dir("./flags/").unwrap();
    for path in flag_files {
        output_data.push(build_flag_info(path.unwrap().path()));
    }

    let mut output_file = File::create("flag_edges.json").unwrap();
    output_file.write_all(serde_json::to_string(&output_data).unwrap().as_bytes()).unwrap();

    println!("{:?}", output_data);
}
