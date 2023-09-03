use std::fs;

use image::DynamicImage;
use image::io::Reader as ImageReader;

use crate::network::{http_get_image, http_get_text};


pub fn load_text(url: &String) -> String {
    let file_contents: String;

    if url.starts_with("file://") {
        let file_path = url[7..] //remove the "file://" prefix
                        .to_owned();

        //TODO: below we need better error handling (return a not found, we need that for network too, and images too)
        file_contents = fs::read_to_string(file_path).expect("Something went wrong reading the file");
    } else {
        file_contents = http_get_text(url);
    }

    return file_contents;
}


pub fn load_image(url: &String) -> DynamicImage {
    let image: DynamicImage;

    if url.starts_with("file://") {
        let file_path = url[7..] //remove the "file://" prefix
                        .to_owned();

        //TODO: this needs better error handling (return a not found, shoul dbe done for network too)
        let file_data = ImageReader::open(file_path).expect("Could not open image");
        image = file_data.decode().expect("decoding the image failed");

    } else {
        image = http_get_image(url);
    }

    return image;
}
