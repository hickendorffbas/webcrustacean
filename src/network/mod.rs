use std::fmt;

use image::DynamicImage;

use crate::debug::debug_log_warn;
use crate::network::url::Url;

pub mod url;
#[cfg(test)] mod tests;


const UA_FIREFOX_WINDOWS: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/118.0";


#[derive(Clone, Debug)] //note: debug here is not conditional on the debug build attribute, because we also need to print errors in release mode
pub struct ResourceNotLoadedError(String); //TODO: eventually we should be more specific, i.e. NetworkError, DecodingError etc.
impl fmt::Display for ResourceNotLoadedError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ResourceNotLoadedError: could not load {}", self.0)
    }
}


pub fn http_get_text(url: &Url) -> Result<String, ResourceNotLoadedError>  {
    //TODO: not sure if I really need a seperate one for text, should I not just never call the .text() method from reqwest,
    //      and just decode myself based on the situation?
    //TODO: in any case we need to de-duplicate between http_get_text() and http_get_image()

    //TODO: should I cache the client somewhere for performance?
    let client = reqwest::blocking::Client::builder()
        .user_agent(UA_FIREFOX_WINDOWS)  //TODO: make this configurable, and use an actual webcrustacean useragent normally
        .build().unwrap();

    let bytes_result = client.get(url.to_string()).send();

    if !bytes_result.is_ok() {
        return Err(ResourceNotLoadedError(url.to_string()));
    }

    let text_result = bytes_result.unwrap().text();

    if text_result.is_ok() {
        return Ok(text_result.unwrap());
    } else {
        debug_log_warn(format!("Could not load text: {}", url.to_string()));
        return Err(ResourceNotLoadedError(url.to_string()));
    }

}


//TODO: there is too much duplication here with the get case...
pub fn http_post(url: &Url, body: String) -> Result<String, ResourceNotLoadedError>  {

    //TODO: should I cache the client somewhere for performance?
    let client = reqwest::blocking::Client::builder()
        .user_agent(UA_FIREFOX_WINDOWS)  //TODO: make this configurable, and use an actual webcrustacean useragent normally
        .build().unwrap();

    let bytes_result = client.post(url.to_string()).body(body).send();

    if !bytes_result.is_ok() {
        return Err(ResourceNotLoadedError(url.to_string()));
    }

    //TODO: we might receive other things than text, so split this out to another method
    let text_result = bytes_result.unwrap().text();

    if text_result.is_ok() {
        return Ok(text_result.unwrap());
    } else {
        debug_log_warn(format!("Could not load text: {}", url.to_string()));
        return Err(ResourceNotLoadedError(url.to_string()));
    }
}



//TODO: eventually this should be a http_get_binary, and the image stuff should be seperated out, because we will load other binary resources.
pub fn http_get_image(url: &Url) -> Result<DynamicImage, ResourceNotLoadedError> {

    //TODO: should I cache the client somewhere for performance?
    let client = reqwest::blocking::Client::builder()
        .user_agent(UA_FIREFOX_WINDOWS)  //TODO: make this configurable, and use an actual webcrustacean useragent normally
        .build().unwrap();

    let response = client.get(url.to_string()).send().unwrap();

    let bytes_result = response.bytes();

    if !bytes_result.is_ok() {
        return Err(ResourceNotLoadedError(url.to_string()));
    }

    //TODO: we would like to return the bytes, for now making an image though, eventually this should be somewhere else (in the resource loader maybe?)
    let image_result = image::load_from_memory(&bytes_result.unwrap());

    if image_result.is_ok() {
        return Ok(image_result.unwrap());
    } else {
        return Err(ResourceNotLoadedError(url.to_string()));
    }

}
