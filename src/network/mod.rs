use std::fmt;

use image::DynamicImage;
use reqwest::blocking::Client;

use crate::debug::debug_log_warn;


#[cfg(test)] mod tests;


#[derive(Debug, Clone)]
pub struct ResourceNotLoadedError(Url); //TODO: eventually we should be more specific, i.e. NetworkError, DecodingError etc.
impl fmt::Display for ResourceNotLoadedError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ResourceNotLoadedError: could not load {}", self.0.to_string())
    }
}


#[derive(PartialEq, Debug, Clone)]
pub struct Url {
    pub scheme: String,
    pub domain: String,
    pub path: String,
    //TODO: eventually should contain query params, port and fragment
}
impl Url {
    pub fn from(url_str: &String) -> Url {
        let mut url_to_parse = &url_str[..];

        //note that below we search for // rather then :// to support scheme-relative url's
        let opt_scheme_end = url_to_parse.find("//");

        let scheme = if opt_scheme_end.is_some() {
            let scheme_part = if opt_scheme_end.unwrap() == 0 {
                //we have a scheme-relative url
                String::new()
            } else {
                //we have a url with the scheme specified
                url_to_parse[..opt_scheme_end.unwrap()-1].to_lowercase()  // we do -1 to remove the ":" after the scheme
            };
            let domain_start = opt_scheme_end.unwrap() + 2;
            url_to_parse = &url_to_parse[domain_start..];
            scheme_part
        } else {
            String::from("http")
        };

        let domain = if scheme != "file" {
            let opt_path_start = url_to_parse.find("/");

            let domain = if opt_path_start.is_some() {
                let domain_part = url_to_parse[..opt_path_start.unwrap()].to_owned();
                url_to_parse = &url_to_parse[opt_path_start.unwrap()+1..];
                domain_part
            } else {
                let domain_part = url_to_parse.to_owned();
                url_to_parse = &url_to_parse[url_to_parse.len()..];
                domain_part
            };
            domain
        } else {
            String::new()
        };

        //TODO: the below is not correct, but we don't parse query params, ports and fragments yet
        let path = url_to_parse.to_owned();

        return Url {scheme, domain, path};
    }

    pub fn from_possible_relative_url(main_url: &Url, maybe_relative_url: &String) -> Url {
        if maybe_relative_url.chars().next() == Some('/') {

            if &maybe_relative_url[0..2] == "//" {
                let partial_url = Url::from(maybe_relative_url);
                Url { scheme: main_url.scheme.clone(), domain: partial_url.domain.clone(), path: partial_url.path}
            } else {
                Url { scheme: main_url.scheme.clone(), domain: main_url.domain.clone(), path: maybe_relative_url[1..].to_owned()}
            }
        } else {
            //The url is not relative
            Url::from(maybe_relative_url)
        }
    }

    pub fn to_string(&self) -> String {
        let mut full_string = String::new();

        full_string.push_str(&self.scheme);
        full_string.push_str("://");
        full_string.push_str(&self.domain);
        if !self.domain.is_empty() {
            full_string.push_str("/");
        }
        full_string.push_str(&self.path);

        return full_string;
    }

    pub fn file_extension(&self) -> Option<String> {
        let dot_position = self.path.find('.');
        if dot_position.is_none() {
            return None;
        }
        let extension_start = dot_position.unwrap() + 1;
        return Some(self.path[extension_start..].to_lowercase());
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
