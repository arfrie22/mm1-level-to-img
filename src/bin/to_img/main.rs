use std::{fs::File, path::Path};

use image::RgbImage;
use indicatif::{ProgressBar, ProgressStyle};
use mm1_level_parser::course::Course;
use mm1_level_to_img::image_from_level;

fn create_image_from_course(course: &Course) -> RgbImage {
    let mut img = RgbImage::new(256, 256);

    image_from_level(&course.level, &mut img, 0);

    img
}

fn parse_archive(path: &Path) -> Result<Course, mm1_level_parser::Error> {
    let file = File::open(path).unwrap();
    let decoder = zstd::Decoder::new(file).unwrap();
    let mut archive = tar::Archive::new(decoder);

    let course = Course::from_tar(&mut archive)?;

    let img = create_image_from_course(&course);

    let output_path = path
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .replace(".tar.zst", ".png");

    img.save(format!("output/{}", output_path)).unwrap();

    Ok(course)
}

fn main() {
    std::fs::create_dir_all("output").unwrap();
    let levels_dir = Path::new("levels");

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
            match parse_archive(&path) {
                Ok(_) => {}
                Err(_) => {
                    continue;
                }
            };
        }

        bar.inc(1);
    }
}
