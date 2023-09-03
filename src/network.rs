use image::DynamicImage;
use reqwest::blocking::Client;


pub fn http_get_text(url: &String) -> String {
    //TODO: not sure if I really need a seperate one for text, should I not just never call the .text() method from reqwest,
    //      and just decode myself based on the situation?

    let client = Client::new();
    let response = client.get(url).send().unwrap();
    return response.text().unwrap();

}

//TODO: eventually this should be a http_get_binary, and the image stuff should be seperated out, because we will load other binary resources.
pub fn http_get_image(url: &String) -> DynamicImage {

    let client = Client::new();  //TODO: should I cache the client somewhere for performance?
    let response = client.get(url).send().unwrap();

    let bytes = response.bytes().unwrap();

    //TODO: we would like to return the bytes, for now making an image though, for testing:
    let image = image::load_from_memory(&bytes).unwrap();

    return image;
}
