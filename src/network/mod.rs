use std::collections::HashMap;

use chrono::{Duration, Utc};
use image::RgbaImage;
use reqwest::blocking::{RequestBuilder, Response};
use reqwest::Error;

use crate::network::url::Url;
use crate::resource_loader::{
    CookieEntry,
    ResourceRequestResult,
    ResourceRequestResultSuccess,
};

pub mod url;
#[cfg(test)] mod tests;


#[allow(unused)] pub const UA_FIREFOX_UBUNTU: &str = "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:141.0) Gecko/20100101 Firefox/141.0";
#[allow(unused)] pub const UA_FIREFOX_WINDOWS: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:141.0) Gecko/20100101 Firefox/141.0";
#[allow(unused)] pub const UA_CHROME_UBUNTU: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/139.0.0.0 Safari/537.36";
#[allow(unused)] pub const UA_WEBCRUSTACEAN_UBUNTU: &str =  concat!("WebCrustacean ", env!("CARGO_PKG_VERSION"));


pub fn http_get_text(url: &Url, cookies: &HashMap<String, String>) -> ResourceRequestResult<String>  {

    let response = http_get(url, cookies);
    if !response.is_ok() {
        return ResourceRequestResult::NotFound;
    }
    let response = response.unwrap();
    let new_cookies = extract_new_cookies(&response);

    let text_result = response.text();

    if text_result.is_ok() {
        return ResourceRequestResult::Success(ResourceRequestResultSuccess { body: text_result.unwrap(), new_cookies: new_cookies });
    } else {
        return ResourceRequestResult::NotFound;
    }
}


pub fn http_get_image(url: &Url, cookies: &HashMap<String, String>) -> ResourceRequestResult<RgbaImage> {

    let response = http_get(url, cookies);
    if !response.is_ok() {
        return ResourceRequestResult::NotFound;
    }
    let response = response.unwrap();
    let new_cookies = extract_new_cookies(&response);

    let image_result = image::load_from_memory(&response.bytes().unwrap());

    if image_result.is_ok() {
        return ResourceRequestResult::Success(ResourceRequestResultSuccess { body: image_result.unwrap().to_rgba8(), new_cookies: new_cookies });
    } else {
        return ResourceRequestResult::NotFound;
    }
}


pub fn http_post_for_text(url: &Url, body: String, cookies: &HashMap<String, String>) -> ResourceRequestResult<String> {

    let response = http_post(url, body, cookies);

    if !response.is_ok() {
        return ResourceRequestResult::NotFound;
    }
    let response = response.unwrap();
    let new_cookies = extract_new_cookies(&response);
    let text_result = response.text();

    if text_result.is_ok() {
        return ResourceRequestResult::Success(ResourceRequestResultSuccess { body: text_result.unwrap(), new_cookies: new_cookies });
    } else {
        return ResourceRequestResult::NotFound;
    }
}


fn http_get(url: &Url, cookies: &HashMap<String, String>) -> Result<Response, Error> {

    //TODO: should I cache the client somewhere for performance?
    let client = reqwest::blocking::Client::builder()
        .user_agent(crate::USER_AGENT)
        .build().unwrap();

    let get_operation = add_cookies(client.get(url.to_string()), cookies);
    let response = get_operation.send();

    return response;
}


fn http_post(url: &Url, body: String, cookies: &HashMap<String, String>) -> Result<Response, Error> {

    //TODO: should I cache the client somewhere for performance?
    let client = reqwest::blocking::Client::builder()
        .user_agent(crate::USER_AGENT)
        .build().unwrap();

    let body_len = body.len();

    let response = add_cookies(client.post(url.to_string()).body(body), cookies)
        .header("Content-Length", body_len.to_string())
        .header("Content-Type", "application/x-www-form-urlencoded")  //TODO: this is not correct for file uploads
        .send();

    return response;
}


fn add_cookies(mut builder: RequestBuilder, cookies: &HashMap<String, String>) -> RequestBuilder {
    for (key, value) in cookies {
        let mut cookie_value = key.clone();
        cookie_value.push('=');
        cookie_value.push_str(value);
        builder = builder.header("Cookie", cookie_value);
    }

    return builder;
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

