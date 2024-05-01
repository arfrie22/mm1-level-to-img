use std::collections::HashMap;

use chrono::Local;
use image::{Rgb, RgbImage};
use mm1_level_parser::{level::{AutoScroll, CourseTheme, GameMode, Level}, objects::Object};
use random_word::Lang;

pub fn colorize_u16(data: u16) -> Rgb<u8> {
    let r = ((data & 0x3F) << 2) as u8;
    let g = ((data >> 4) & 0xFC) as u8;
    let b = ((data >> 10) & 0xFC) as u8;
    Rgb([r, g, b])
}

pub fn uncolorize_u16(color: &Rgb<u8>) -> u16 {
    let r = color.0[0] as u16;
    let g = color.0[1] as u16;
    let b = color.0[2] as u16;
    (r >> 2) | ((g & 0xFC) << 4) | ((b & 0xFC) << 10)
}

pub fn colorize_u8(data: u8) -> Rgb<u8> {
    colorize_u16((data as u16) << 8)
}

pub fn uncolorize_u8(color: &Rgb<u8>) -> u8 {
    (uncolorize_u16(color) >> 8) as u8
}

pub fn u16_from_i8(data: i8) -> u16 {
    (data as u8) as u16
}

pub fn image_from_level(level: &Level, img: &mut RgbImage, y_offset: u32) {
    let mut x = 0;
    img.put_pixel(x, y_offset, colorize_u8(u8::from(level.game_mode) << 6));
    x += 1;

    img.put_pixel(x, y_offset, colorize_u8(u8::from(level.course_theme) << 5));
    x += 1;

    img.put_pixel(x, y_offset, colorize_u16(level.time_limit));
    x += 1;

    img.put_pixel(x, y_offset, colorize_u8(u8::from(level.auto_scroll) << 6));
    x += 1;

    img.put_pixel(x, y_offset, colorize_u8(level.flags.into()));
    x += 1;

    for b in level.width.to_le_bytes().iter() {
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

        if x > 240 || y > 27 {
            continue;
        }

        let obj_type = (u16_from_i8(obj.object_type) << 8) | u16_from_i8(obj.transformation_id);
        img.put_pixel(x, y + y_offset + 1, colorize_u16(obj_type));
        img.put_pixel(
            x,
            y + y_offset + 1 + 27,
            colorize_u16((obj.z_position >> 16) as u16),
        );
        img.put_pixel(
            x,
            y + y_offset + 1 + 27 * 2,
            colorize_u16((obj.z_position & 0xFFFF) as u16),
        );
        img.put_pixel(
            x,
            y + y_offset + 1 + 27 * 3,
            colorize_u16((obj.object_flags >> 16) as u16),
        );
        img.put_pixel(
            x,
            y + y_offset + 1 + 27 * 4,
            colorize_u16((obj.object_flags & 0xFFFF) as u16),
        );
        let size = (u16_from_i8(obj.width) << 8) | u16_from_i8(obj.height);
        img.put_pixel(
            x,
            y + y_offset + 1 + 27 * 5,
            colorize_u16(size)
        );

        let child_obj_type = (u16_from_i8(obj.child_object_type) << 8) | u16_from_i8(obj.child_object_transformation_id);
        img.put_pixel(x, y + y_offset + 1 + 27 * 6, colorize_u16(child_obj_type));
        img.put_pixel(
            x,
            y + y_offset + 1 + 27 * 7,
            colorize_u16((obj.child_object_flags >> 16) as u16),
        );
        img.put_pixel(
            x,
            y + y_offset + 1 + 27 * 8,
            colorize_u16((obj.child_object_flags & 0xFFFF) as u16),
        );
    }
}

pub fn level_from_img(img: &RgbImage, y_offset: u32) -> Level {
    let mut x = 0;

    let game_mode = GameMode::try_from(uncolorize_u8(img.get_pixel(x, y_offset)) >> 6).unwrap_or_default();
    x += 1;

    let course_theme = CourseTheme::try_from(uncolorize_u8(img.get_pixel(x, y_offset)) >> 5).unwrap_or_default();
    x += 1;

    let time_limit = uncolorize_u16(img.get_pixel(x, y_offset));

    x += 1;

    let auto_scroll = AutoScroll::try_from(uncolorize_u8(img.get_pixel(x, y_offset)) >> 6).unwrap_or_default();
    x += 1;

    let flags = uncolorize_u8(img.get_pixel(x, y_offset));
    x += 1;

    let mut width = 0u32;
    for i in 0..4 {
        width |= (uncolorize_u8(img.get_pixel(x, y_offset)) as u32) << (i * 8);
        x += 1;
    }

    let mut objects = Vec::new();
    const SCALE: u32 = 160;
    for y in 0..27 {
        for x in 0..240 {
            let x_pos = x * SCALE;
            let y_pos = y as i16 * SCALE as i16;
            
            let obj_type = uncolorize_u16(img.get_pixel(x, y + y_offset + 1));
            let object_type = (obj_type >> 8) as i8;
            if object_type == 0 {
                continue;
            }
            let transformation_id = obj_type as i8;

            let z_position = (uncolorize_u16(img.get_pixel(x, y + y_offset + 1 + 27)) as u32) << 16
                | (uncolorize_u16(img.get_pixel(x, y + y_offset + 1 + 27 * 2)) as u32);

            let object_flags = (uncolorize_u16(img.get_pixel(x, y + y_offset + 1 + 27 * 3)) as u32) << 16
                | (uncolorize_u16(img.get_pixel(x, y + y_offset + 1 + 27 * 4)) as u32);

            let size = uncolorize_u16(img.get_pixel(x, y + y_offset + 1 + 27 * 5));
            let width = (size >> 8) as i8;  
            let height = size as i8;

            let child_obj_type = uncolorize_u16(img.get_pixel(x, y + y_offset + 1 + 27 * 6));
            let child_object_type = (child_obj_type >> 8) as i8;
            let child_object_transformation_id = child_obj_type as i8;

            let child_object_flags = (uncolorize_u16(img.get_pixel(x, y + y_offset + 1 + 27 * 7)) as u32) << 16
                | (uncolorize_u16(img.get_pixel(x, y + y_offset + 1 + 27 * 8)) as u32);

            let object = Object {
                x_position: x_pos,
                y_position: y_pos,
                z_position,
                object_type,
                transformation_id,
                object_flags,
                width,
                height,
                child_object_type,
                child_object_transformation_id,
                child_object_flags,
                extended_object_data: 0,
                link_id: -1,
                effect_index: -1,
            };

            objects.push(object);
        }
    }

    let level_name = random_word::gen(Lang::En).to_owned() + "-" + random_word::gen(Lang::En);

    Level {
        course_theme,
        time_limit,
        auto_scroll,
        flags,
        width,
        objects,
        version: 0,
        creation_time: Local::now().naive_local(),
        level_name,
        game_mode,
        mii_data: [0; 96],
        sound_effects: Vec::new(),
    }
}