use std::collections::HashMap;

use chrono::{Duration, Utc};
use image::RgbaImage;
use reqwest::blocking::Response;

use crate::network::url::Url;
use crate::resource_loader::{
    CookieEntry,
    ResourceRequestResult,
    ResourceRequestResultSuccess,
};

pub mod url;
#[cfg(test)] mod tests;


const UA_FIREFOX_UBUNTU: &str = "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:137.0) Gecko/20100101 Firefox/137.0";


pub fn http_get_text(url: &Url, cookies: &HashMap<String, String>) -> ResourceRequestResult<String>  {
    //TODO: not sure if I really need a seperate one for text, should I not just never call the .text() method from reqwest,
    //      and just decode myself based on the situation?
    //TODO: in any case we need to de-duplicate between http_get_text() and http_get_image()

    //TODO: should I cache the client somewhere for performance?
    let client = reqwest::blocking::Client::builder()
        .user_agent(UA_FIREFOX_UBUNTU)  //TODO: make this configurable, and use an actual webcrustacean useragent normally
        .build().unwrap();

    let mut get_operation = client.get(url.to_string());

    //TODO: we need this in a more centralized place (a single one for GET, also non-text)
    for (key, value) in cookies {
        let mut cookie_value = key.clone();
        cookie_value.push('=');
        cookie_value.push_str(value);
        get_operation = get_operation.header("Cookie", cookie_value);
    }

    let bytes_result = get_operation.send();

    if !bytes_result.is_ok() {
        return ResourceRequestResult::NotFound;
    }
    let response = bytes_result.unwrap();

    let new_cookies = extract_new_cookies(&response);
    let text_result = response.text();

    if text_result.is_ok() {
        return ResourceRequestResult::Success(ResourceRequestResultSuccess { body: text_result.unwrap(), new_cookies: new_cookies });
    } else {
        return ResourceRequestResult::NotFound;
    }
}


//TODO: there is too much duplication here with the get case...
pub fn http_post(url: &Url, body: String) -> ResourceRequestResult<String>  {

    //TODO: should I cache the client somewhere for performance?
    let client = reqwest::blocking::Client::builder()
        .user_agent(UA_FIREFOX_UBUNTU)  //TODO: make this configurable, and use an actual webcrustacean useragent normally
        .build().unwrap();

    let body_len = body.len();

    let bytes_result = client.post(url.to_string()).body(body)

        .header("Content-Length", body_len.to_string())
        .header("Content-Type", "application/x-www-form-urlencoded")  //TODO: not sure if this is always correct for all posts
                                                                      //   (probably not in general, but for forms it might be)

        .send();

    if !bytes_result.is_ok() {
        return ResourceRequestResult::NotFound;
    }
    let response = bytes_result.unwrap();
    let new_cookies = extract_new_cookies(&response);

    //TODO: we might receive other things than text, so split this out to another method
    let text_result = response.text();


    if text_result.is_ok() {
        //TODO: set the below new_cookies based on the response
        return ResourceRequestResult::Success(ResourceRequestResultSuccess { body: text_result.unwrap(), new_cookies: new_cookies });
    } else {
        return ResourceRequestResult::NotFound;
    }
}


//TODO: eventually this should be a http_get_binary, and the image stuff should be seperated out, because we will load other binary resources.
pub fn http_get_image(url: &Url) -> ResourceRequestResult<RgbaImage> {

    //TODO: should I cache the client somewhere for performance?
    let client = reqwest::blocking::Client::builder()
        .user_agent(UA_FIREFOX_UBUNTU)  //TODO: make this configurable, and use an actual webcrustacean useragent normally
        .build().unwrap();

    let response = client.get(url.to_string()).send().unwrap();
    let new_cookies = extract_new_cookies(&response);

    let bytes_result = response.bytes();

    if !bytes_result.is_ok() {
        return ResourceRequestResult::NotFound;
    }
    let response = bytes_result.unwrap();

    //TODO: we would like to return the bytes, for now making an image though, eventually this should be somewhere else (in the resource loader maybe?)
    let image_result = image::load_from_memory(&response);

    if image_result.is_ok() {
        return ResourceRequestResult::Success(ResourceRequestResultSuccess { body: image_result.unwrap().to_rgba8(), new_cookies: new_cookies });
    } else {
        return ResourceRequestResult::NotFound;
    }
}


fn extract_new_cookies(response: &Response) -> HashMap<String, CookieEntry> {
    let mut  new_cookies = HashMap::new();

    for (header_name, header_value) in response.headers() {
        if header_name == "set-cookie" {
            let header_value = header_value.to_str().unwrap();

            let mut cookie_name = String::new();
            let mut cookie_value = String::new();
            let mut equal_seen = false;
            let mut in_flags = false;

            for char in header_value.chars() {
                if char == '=' {
                    equal_seen = true;
                } else if char == ';' {
                    in_flags = true;
                } else if !equal_seen && !in_flags {
                    cookie_name.push(char);
                } else if !in_flags {
                    cookie_value.push(char);
                }

                //TODO: parse flags here
            }

            new_cookies.insert(cookie_name, CookieEntry { value: cookie_value, expiry_time: Utc::now() + Duration::days(1) });
        }
    }

    return new_cookies;
}

