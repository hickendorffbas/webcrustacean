use crate::network::Url;



#[test]
fn test_basic_url_parsing() {
    assert_eq!(Url::from(&String::from("http://www.google.com/page")), build_url("http", "www.google.com", "page"));
    assert_eq!(Url::from(&String::from("http://www.google.com/")), build_url("http", "www.google.com", ""));
    assert_eq!(Url::from(&String::from("http://www.google.com")), build_url("http", "www.google.com", ""));
}


#[test]
fn test_from_possible_relative_url() {
    let current_url = Url::from(&String::from("http://www.google.com/page"));
    assert_eq!(Url::from_possible_relative_url(&current_url, &String::from("/other_page")), build_url("http", "www.google.com", "other_page"));
    assert_eq!(Url::from_possible_relative_url(&current_url, &String::from("/other_folder/page")), build_url("http", "www.google.com", "other_folder/page"));
    assert_eq!(Url::from_possible_relative_url(&current_url, &String::from("//google.com/other_folder/page")), build_url("http", "google.com", "other_folder/page"));

    let current_url = Url::from(&String::from("https://www.google.com/page"));
    assert_eq!(Url::from_possible_relative_url(&current_url, &String::from("//google.com/other_folder/page")), build_url("https", "google.com", "other_folder/page"));
}


#[test]
fn test_technically_invalid_url_parsing() {
    assert_eq!(Url::from(&String::from("www.google.com")), build_url("http", "www.google.com", ""));
}


#[test]
fn test_file_url_parsing() {
    assert_eq!(Url::from(&String::from("file://some/good/file.html")), build_url("file", "", "some/good/file.html"));
}


fn build_url(scheme: &str, domain: &str, path: &str) -> Url {
    return Url { scheme: scheme.to_owned(), domain: domain.to_owned(), path: path.to_owned() };
}
