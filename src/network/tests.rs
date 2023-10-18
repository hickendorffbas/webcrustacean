use crate::network::Url;



#[test]
fn test_basic_url_parsing() {
    assert_eq!(Url::from(&String::from("http://www.google.com/page")), build_url("http", "www.google.com", &vec![String::from("page")]));
    assert_eq!(Url::from(&String::from("http://www.google.com/")), build_url("http", "www.google.com", &vec![String::new()]));
    assert_eq!(Url::from(&String::from("http://www.google.com")), build_url("http", "www.google.com", &Vec::new()));
}


#[test]
fn test_from_possible_relative_url() {
    let current_url = Url::from(&String::from("http://www.google.com/page1"));

    assert_eq!(Url::from_base_url(&String::from("/other_page"), Some(&current_url)), build_url("http", "www.google.com", &vec![String::from("other_page")]));
    assert_eq!(Url::from_base_url(&String::from("/other_folder/page"), Some(&current_url)),
               build_url("http", "www.google.com", &vec![String::from("other_folder"), String::from("page")]));
    assert_eq!(Url::from_base_url(&String::from("//www.google.com/other_folder/page"), Some(&current_url)),
               build_url("http", "www.google.com", &vec![String::from("other_folder"), String::from("page")]));

    let current_url = Url::from(&String::from("https://www.google.com/page"));
    assert_eq!(Url::from_base_url(&String::from("//google.com/other_folder/page"), Some(&current_url)),
               build_url("https", "google.com", &vec![String::from("other_folder"), String::from("page")]));
}


#[test]
fn test_file_url_parsing() {
    assert_eq!(Url::from(&String::from("file:///some/good/file.html")),
               build_url("file", "", &vec![String::from("some"), String::from("good"), String::from("file.html")]));
}


#[test]
fn test_data_url_parsing() {
    assert_eq!(Url::from(&String::from("data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAB4A")),
               build_url("data", "", &vec!["image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAB4A".to_owned()]));
}


fn build_url(scheme: &str, host: &str, path: &Vec<String>) -> Url {
    return Url { scheme: scheme.to_owned(), host: host.to_owned(), path: path.clone(),
                 username: String::new(), password: String::new(), port: String::new(), query: String::new(), fragment: String::new(), blob: String::new() };
}
