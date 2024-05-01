use std::{io::{Error, Write}, path::{Path, PathBuf}};

use image::RgbImage;
use indicatif::{ProgressBar, ProgressStyle};

use mm1_level_parser::level::Level;
use mm1_level_to_img::level_from_img;

fn create_image_from_course(path: &PathBuf) -> Result<Level, Error> {
    let img = image::open(path).map_err(|e| Error::new(std::io::ErrorKind::Other, format!("Failed to open image: {}", e)),)?.to_rgb8();

    let level = level_from_img(&img, 0);
    let data = level.to_bytes().map_err(|e| Error::new(std::io::ErrorKind::Other, format!("Failed to convert level to bytes: {:?}", e)))?;

    let output_path = path
        .file_name()
        .ok_or(Error::new(std::io::ErrorKind::Other, "No file name"))?
        .to_str()
        .unwrap()
        .replace(".png", ".cdt");

    let output_path = Path::new("generate").join(output_path);

    let mut file = std::fs::File::create(output_path).map_err(|e| Error::new(std::io::ErrorKind::Other, format!("Failed to create file: {}", e)))?;
    file.write_all(&data).map_err(|e| Error::new(std::io::ErrorKind::Other, format!("Failed to write data: {}", e)))?;
    
    Ok(level)
}
fn main() {
    std::fs::create_dir_all("generate").unwrap();
    let levels_dir = Path::new("output-2");

    let bar = ProgressBar::new(levels_dir.read_dir().unwrap().count() as u64);
    bar.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] ({pos}/{len}, ETA {eta})",
        )
        .unwrap(),
    );

    for entry in levels_dir.read_dir().unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            match create_image_from_course(&path) 
            {
                Ok(_) => {},
                Err(e) => {
                    println!("Failed to parse {:?}", e);
                    continue;
                }
            };
        }

        bar.inc(1);
    }
}
