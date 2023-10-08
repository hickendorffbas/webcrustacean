use crate::network::Url;



#[test]
fn test_basic_url_parsing() {
    assert_eq!(Url::from(&String::from("http://www.google.com/page")), build_url("http", "www.google.com", "page"));
    assert_eq!(Url::from(&String::from("http://www.google.com/")), build_url("http", "www.google.com", ""));
    assert_eq!(Url::from(&String::from("http://www.google.com")), build_url("http", "www.google.com", ""));
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
