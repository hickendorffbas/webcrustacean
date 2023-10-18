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
    if url.scheme == "file" {
        let mut local_path = String::from("//");
        local_path.push_str(&url.path.join("/"));
        let read_result = fs::read_to_string(local_path);
        if read_result.is_err() {
            debug_log_warn(format!("Could not load text: {}", url.to_string()));
            return String::new();
        }

        return read_result.unwrap();
    }

    let file_content_result = http_get_text(url);

    if file_content_result.is_err() {
        debug_log_warn(format!("Could not load text: {}", url.to_string()));
        return String::new();
    }

    return file_content_result.unwrap();
}


pub fn load_image(url: &Url) -> DynamicImage {
    if url.scheme == "file" {
        let mut local_path = String::from("//");
        local_path.push_str(&url.path.join("/"));
        let read_result = ImageReader::open(local_path);
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
    if url.scheme == "data".to_owned() {
        //data scheme is currently not implemented
        debug_log_warn(format!("the data: scheme is not supported currently: {}", url.to_string()));
        return fallback_image();
    }

    println!("loading {}", url.to_string());

    let image_result = http_get_image(url);
    if image_result.is_err() {
        debug_log_warn(format!("Could not load image: {}", url.to_string()));
        return fallback_image();
    }

    return image_result.unwrap();
}


fn fallback_image() -> DynamicImage {
    //TODO: this should become one of those "broken image"-images
    return DynamicImage::new_rgb32f(1, 1);
}