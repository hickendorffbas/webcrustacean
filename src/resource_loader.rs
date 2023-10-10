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

    if url.scheme == "file" {
        let read_result = ImageReader::open(&url.path);
        if read_result.is_err() {
            debug_log_warn(format!("Could not load image: {}", url.to_string()));
            return fallback_image();
        }

        let file_data = read_result.unwrap();
        return file_data.decode().expect("decoding the image failed");
    }

    let extension = url.file_extension();
    if extension.is_some() && extension.unwrap() == "svg".to_owned() {
        //svg is currently not implemented
        debug_log_warn(format!("Svg's are not supported currently: {}", url.to_string()));
        return fallback_image();
    }

    println!("url: {}", url.to_string());

    //TODO: this needs error handling
    return http_get_image(url);
}


fn fallback_image() -> DynamicImage {
    //TODO: this should become one of those "broken image"-images
    return DynamicImage::new_rgb32f(1, 1);
}