
use reqwest::blocking::Client;


pub fn http_get(url: String) -> String {

    let client = Client::new();
    let result = client.get(url).send().unwrap();
    return result.text().unwrap();

}

