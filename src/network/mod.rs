use std::fmt;

use image::DynamicImage;
use reqwest::blocking::Client;

use crate::debug::debug_log_warn;
use crate::network::url::Url;

pub mod url;
#[cfg(test)] mod tests;


#[derive(Clone)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct ResourceNotLoadedError(Url); //TODO: eventually we should be more specific, i.e. NetworkError, DecodingError etc.
impl fmt::Display for ResourceNotLoadedError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ResourceNotLoadedError: could not load {}", self.0.to_string())
    }
}


pub fn http_get_text(url: &Url) -> Result<String, ResourceNotLoadedError>  {
    //TODO: not sure if I really need a seperate one for text, should I not just never call the .text() method from reqwest,
    //      and just decode myself based on the situation?
    //TODO: in any case we need to de-duplicate between http_get_text() and http_get_image()

    let client = Client::new();  //TODO: should I cache the client somewhere for performance?

    let bytes_result = client.get(url.to_string()).send();

    if !bytes_result.is_ok() {
        return Err(ResourceNotLoadedError(url.clone()));
    }

    let text_result = bytes_result.unwrap().text();

    if text_result.is_ok() {
        return Ok(text_result.unwrap());
    } else {
        debug_log_warn(format!("Could not load text: {}", url.to_string()));
        return Err(ResourceNotLoadedError(url.clone()));
    }

}


//TODO: eventually this should be a http_get_binary, and the image stuff should be seperated out, because we will load other binary resources.
pub fn http_get_image(url: &Url) -> Result<DynamicImage, ResourceNotLoadedError> {

    let client = Client::new();  //TODO: should I cache the client somewhere for performance?
    let response = client.get(url.to_string()).send().unwrap();

    let bytes_result = response.bytes();

    if !bytes_result.is_ok() {
        return Err(ResourceNotLoadedError(url.clone()));
    }

    //TODO: we would like to return the bytes, for now making an image though, eventually this should be somewhere else (in the resource loader maybe?)
    let image_result = image::load_from_memory(&bytes_result.unwrap());

    if image_result.is_ok() {
        return Ok(image_result.unwrap());
    } else {
        debug_log_warn(format!("Could not load image: {}", url.to_string()));
        return Err(ResourceNotLoadedError(url.clone()));
    }

}
