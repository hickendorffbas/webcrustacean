use std::fs;

use image::DynamicImage;
use image::io::Reader as ImageReader;

use crate::debug::debug_log_warn;
use crate::network::{
    Url,
    http_get_image,
    http_get_text,
};


pub fn load_text(url: &Url) -> String {
    let file_contents: String;

    if url.scheme == "file" {
        let read_result = fs::read_to_string(&url.path);
        if read_result.is_err() {
            debug_log_warn(format!("Could not load text: {}", url.to_string()));
            return String::new();
        }

        file_contents = read_result.unwrap();

    } else {
        //TODO: this needs error handling
        file_contents = http_get_text(url);
    }

    return file_contents;
}


pub fn load_image(url: &Url) -> DynamicImage {
    let image: DynamicImage;

    if url.scheme == "file" {
        let read_result = ImageReader::open(&url.path);
        if read_result.is_err() {
            debug_log_warn(format!("Could not load image: {}", url.to_string()));
            return DynamicImage::new_rgb32f(1, 1);
        }

        let file_data = read_result.unwrap();
        image = file_data.decode().expect("decoding the image failed");

    } else {
        //TODO: this needs error handling
        image = http_get_image(url);
    }

    return image;
}
