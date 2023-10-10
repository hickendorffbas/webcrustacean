use image::DynamicImage;
use reqwest::blocking::Client;


#[cfg(test)] mod tests;


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

        let opt_scheme_end = url_to_parse.find("://");

        let scheme = if opt_scheme_end.is_some() {
            let domain_start = opt_scheme_end.unwrap() + 3;
            let scheme_part = url_to_parse[..opt_scheme_end.unwrap()].to_lowercase();
            url_to_parse = &url_to_parse[domain_start..];
            scheme_part
        } else {
            String::from("http") //for now http is the default, not sure if that is always a good idea
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
            Url { scheme: main_url.scheme.clone(), domain: main_url.domain.clone(), path: maybe_relative_url[1..].to_owned()}
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
        full_string.push_str("/");
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


pub fn http_get_text(url: &Url) -> String {
    //TODO: not sure if I really need a seperate one for text, should I not just never call the .text() method from reqwest,
    //      and just decode myself based on the situation?

    let client = Client::new();
    let response = client.get(url.to_string()).send().unwrap();
    return response.text().unwrap();

}


//TODO: eventually this should be a http_get_binary, and the image stuff should be seperated out, because we will load other binary resources.
pub fn http_get_image(url: &Url) -> DynamicImage {

    let client = Client::new();  //TODO: should I cache the client somewhere for performance?
    let response = client.get(url.to_string()).send().unwrap();

    let bytes = response.bytes().unwrap();

    //TODO: we would like to return the bytes, for now making an image though, for testing:
    let image = image::load_from_memory(&bytes).unwrap();

    return image;
}
