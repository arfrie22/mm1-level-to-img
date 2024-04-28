use std::{collections::HashMap, fs::File, path::Path};

use csv::Writer;
use image::{Rgb, RgbImage};
use indicatif::{ProgressBar, ProgressStyle};
use mm1_level_parser::{course::Course, level::Level, objects::Object};

fn colorize_u16(data: u16) -> Rgb<u8> {
    let r = ((data & 0x3F) << 2) as u8;
    let g = ((data >> 4) & 0xFC) as u8;
    let b = ((data >> 10) & 0xFC) as u8;
    Rgb([r, g, b])
}

fn colorize_u8(data: u8) -> Rgb<u8> {
    colorize_u16((data as u16) << 8)
}

fn image_from_level(level: &Level, img: &mut RgbImage, y_offset: u32) {
    let mut x = 0;
    img.put_pixel(x, y_offset, colorize_u8(level.course_theme.into()));
    x += 1;

    for b in level.time_limit.to_le_bytes().iter() {
        img.put_pixel(x, y_offset, colorize_u8(*b));
        x += 1;
    }

    img.put_pixel(x, y_offset, colorize_u8(level.auto_scroll.into()));
    x += 1;

    img.put_pixel(x, y_offset, colorize_u8(level.flags.into()));
    x += 1;

    for b in level.block_width().to_le_bytes().iter() {
        img.put_pixel(x, y_offset, colorize_u8(*b));
        x += 1;
    }

    let mut objects = HashMap::new();

    const SCALE: u32 = 160;
    for obj in level.objects.iter() {
        let x_pos = obj.x_position / SCALE;
        let y_pos = obj.y_position / SCALE as i16;
        let key = (x_pos, y_pos);

        if objects.contains_key(&key) {
            let cur_obj: &Object = objects.get(&key).unwrap();
            if cur_obj.z_position < obj.z_position {
                objects.insert(key, obj.clone());
            }
        } else {
            objects.insert(key, obj.clone());
        }
    }

    for (key, obj) in objects.iter() {
        let x = key.0;
        let y_pos = key.1;
        let y = y_pos as u32;

        let z_block = (obj.z_position & 0xFFFF) as u16;

        if x > 240 || y > 27 {
            continue;
        }

        img.put_pixel(x, y + y_offset + 1, colorize_u8(obj.object_type as u8));
        img.put_pixel(x, y + y_offset + 1 + 27, colorize_u16(z_block));
        img.put_pixel(
            x,
            y + y_offset + 1 + 27 * 2,
            colorize_u16((obj.object_flags >> 16) as u16),
        );
        img.put_pixel(
            x,
            y + y_offset + 1 + 27 * 3,
            colorize_u16((obj.object_flags & 0xFFFF) as u16),
        );
    }
}

fn create_image_from_course(course: &Course) -> RgbImage {
    let mut img = RgbImage::new(256, 256);

    image_from_level(&course.level, &mut img, 5);
    image_from_level(&course.sub_level, &mut img, 110);

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

    img.save(format!("output/{}.png", output_path)).unwrap();

    Ok(course)
}

fn main() {
    std::fs::create_dir_all("output").unwrap();
    let levels_dir = Path::new("levels");
    let mut wtr = Writer::from_path("levels.csv").unwrap();
    wtr.write_record(&[
        "level_name",
        "game_mode",
        "level_theme",
        "sub_level_theme",
        "time_limit",
        "auto_scroll",
        "sub_level_auto_scroll",
        "block_width",
        "sub_level_block_width",
    ])
    .unwrap();

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
            let course = match parse_archive(&path) 
            {
                Ok(course) => course,
                Err(_) => {
                    continue;
                }
            };
            wtr.write_record(&[
                path.file_name().unwrap().to_str().unwrap().replace(".tar.zst", ""),
                (course.level.game_mode as u8).to_string(),
                (course.level.course_theme as u8).to_string(),
                (course.sub_level.course_theme as u8).to_string(),
                course.level.time_limit.to_string(),
                (course.level.auto_scroll as u8).to_string(),
                (course.sub_level.auto_scroll as u8).to_string(),
                course.level.block_width().to_string(),
                course.sub_level.block_width().to_string(),
            ]).unwrap();
        }

        bar.inc(1);
    }
}
