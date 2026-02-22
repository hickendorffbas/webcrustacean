use std::cmp::max;


#[cfg_attr(debug_assertions, derive(Debug))]
enum UrlParsingState {
    SchemeStartState,
    SchemeState,
    NoSchemeState,
    //SpecialRelativeOrAuthorityState, //present in the spec, but not currently implemented
    //PathOrAuthorityState, //present in the spec, but not currently implemented
    RelativeState,
    RelativeSlashState,
    SpecialAuthoritySlashesState,
    SpecialAuthorityIgnoreSlashesState,
    AuthorityState,
    HostState,
    //HostnameState, //present in the spec, but not currently implemented
    PortState,
    FileState,
    FileSlashState,
    FileHostState,
    PathStartState,
    PathState,
    OpaquePathState,
    QueryState,
    FragmentState,
}



#[derive(PartialEq, Clone)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct Url {
    //implementation of https://url.spec.whatwg.org/
    pub scheme: String,
    pub username: String,
    pub password: String,
    pub host: String,
    pub port: String,
    pub path: Vec<String>,
    pub query: String,
    pub fragment: String,
    pub blob: String,
}
impl Url {
    pub fn from(url_str: &String) -> Url {
        Url::from_base_url(url_str, None)
    }

    #[cfg(test)] //currently only used in tests
    pub fn empty() -> Url {
        return Url {
            scheme: String::new(),
            username: String::new(),
            password: String::new(),
            host: String::new(),
            port: String::new(),
            path: Vec::new(),
            query: String::new(),
            fragment: String::new(),
            blob: String::new(),
        }
    }

    pub fn from_base_url(url_str: &String, base_url: Option<&Url>) -> Url {
        let mut state = UrlParsingState::SchemeStartState;
        let mut buffer = String::new();
        let mut pointer: i32 = 0; // the pointer schould point in the string, so normally won't be negative, but we need to decrease it
                                  // in the loop sometimes so it can later be increased to 0 again at the end of the loop. So we need it signed here.
        let mut previous_pointer: i32 = 0;
        let mut url_str_iter = url_str.chars().peekable();

        let mut scheme = String::new();
        let mut username = String::new();
        let mut password = String::new();
        let mut host = String::new();
        let mut port = String::new();
        let mut path = Vec::new();
        let mut query = String::new();
        let mut fragment = String::new();
        let blob = String::new();

        let mut next_char = url_str_iter.next();


        loop {
            match state {
                UrlParsingState::SchemeStartState => {
                    if next_char.is_some() && next_char.unwrap().is_ascii_alphanumeric() {
                        buffer.push(next_char.unwrap().to_ascii_lowercase());
                        state = UrlParsingState::SchemeState;
                    } else {
                        state = UrlParsingState::NoSchemeState;
                        pointer = max(pointer - 1, -1);
                    }
                },

                UrlParsingState::SchemeState => {
                    if next_char.is_some() && next_char.unwrap().is_ascii_alphanumeric() {
                        buffer.push(next_char.unwrap().to_ascii_lowercase());
                    } else if next_char == Some(':') {
                        scheme = buffer;
                        buffer = String::new();

                        if scheme == "file" {
                            state = UrlParsingState::FileState;
                        } else if Url::scheme_is_special(&scheme) {
                            state = UrlParsingState::SpecialAuthoritySlashesState;
                        } else {
                            state = UrlParsingState::OpaquePathState;
                        }
                    } else {
                        buffer = String::new();
                        state = UrlParsingState::NoSchemeState;
                        pointer = -1;
                    }
                },

                UrlParsingState::FileState => {
                    scheme = String::from("file");
                    host = String::new();
                    if next_char == Some('/') {
                        state = UrlParsingState::FileSlashState;
                    } else if base_url.is_some() && base_url.unwrap().scheme == "file" {
                        host = base_url.unwrap().host.clone();
                        path = base_url.unwrap().path.clone();
                        query = base_url.unwrap().query.clone();
                        if next_char == Some('?') {
                            query = String::new();
                            state = UrlParsingState::QueryState;
                        } else if next_char == Some('#') {
                            fragment = String::new();
                            state = UrlParsingState::FragmentState;
                        } else {
                            if next_char.is_some() {
                                query = String::new();
                                if path.len() > 0 {
                                    path.remove(path.len() - 1);
                                }
                                state = UrlParsingState::PathState;
                                pointer = max(pointer - 1, -1);
                            }
                        }
                    } else {
                        state = UrlParsingState::PathState;
                        pointer = max(pointer - 1, -1);
                    }
                }

                UrlParsingState::FileSlashState => {
                    if next_char == Some('/') {
                        state = UrlParsingState::FileHostState;
                    } else {
                        if base_url.is_some() && base_url.unwrap().scheme == "file" {
                            host = base_url.unwrap().host.clone();
                        }
                        state = UrlParsingState::FileState;
                        pointer = max(pointer - 1, -1);
                    }
                },

                UrlParsingState::FileHostState => {
                    if next_char == None || next_char == Some('/') || next_char == Some('\\') || next_char == Some('?') || next_char == Some('#') {
                        pointer = max(pointer - 1, -1);

                        if buffer == "" {
                            host = String::new();
                            state = UrlParsingState::PathStartState;
                        } else {
                            host = Url::parse_host(&buffer);
                            if host == "localhost" {
                                host = String::new();
                            }
                            buffer = String::new();
                            state = UrlParsingState::PathStartState;
                        }

                    } else {
                        buffer.push(next_char.unwrap());
                    }
                },

                UrlParsingState::PathStartState => {
                    if Url::scheme_is_special(&scheme) {
                        state = UrlParsingState::PathState;
                        if next_char != Some('/') {
                            pointer = max(pointer - 1, -1);
                        }
                    } else if next_char == Some('?') {
                        state = UrlParsingState::QueryState;
                    } else if next_char == Some('#') {
                        state = UrlParsingState::FragmentState;
                    } else if next_char.is_some() {
                        state = UrlParsingState::PathState;
                        if next_char != Some('/') {
                            pointer = max(pointer - 1, -1);
                        }
                    }
                },

                UrlParsingState::SpecialAuthorityIgnoreSlashesState => {
                    if next_char != Some('/') && next_char != Some('\\') {
                        state = UrlParsingState::AuthorityState;
                        pointer = max(pointer - 1, -1);
                    } else {
                        todo!(); //this should be an error
                    }
                },

                UrlParsingState::SpecialAuthoritySlashesState =>  {
                    if next_char == Some('/') && url_str_iter.peek() == Some(&'/') {
                        state = UrlParsingState::SpecialAuthorityIgnoreSlashesState;
                        pointer = pointer + 1;
                    } else {
                        todo!(); //this should be an error (but it also moves to a new state?)
                    }
                },

                UrlParsingState::AuthorityState => {
                    if next_char == Some('@') {
                        todo!(); //Auth url's are not yet implemented
                    } else if next_char == None || next_char == Some('/') || next_char == Some('?') || next_char == Some('#') {
                        let buffer_length_plus_one = buffer.len() + 1;
                        pointer = pointer - buffer_length_plus_one as i32;
                        pointer = max(pointer, -1);
                        buffer = String::new();
                        state = UrlParsingState::HostState;
                    } else {
                        buffer.push(next_char.unwrap());
                    }
                },

                UrlParsingState::PathState =>  {
                    if next_char == None || next_char == Some('/') || next_char == Some('?') || next_char == Some('#') {
                        //TODO: check single and double dot path segment
                        path.push(buffer);
                        buffer = String::new();
                        if next_char == Some('?') {
                            state = UrlParsingState::QueryState;
                        } else if next_char == Some('#') {
                            state = UrlParsingState::FragmentState;
                        }
                    } else {
                        buffer.push(next_char.unwrap());
                    }
                },

                UrlParsingState::OpaquePathState => {
                    if next_char == Some('?') {
                        query = String::new();
                        state = UrlParsingState::QueryState;
                    } else if next_char == Some('#') {
                        fragment = String::new();
                        state = UrlParsingState::FragmentState;
                    } else if next_char != None {
                        //TODO: there is something about encoding in the spec that we don't do yet
                        if path.is_empty() {
                            path.push(String::new());
                        }
                        path.last_mut().unwrap().push(next_char.unwrap());
                    }
                },

                UrlParsingState::HostState => {
                    if next_char == Some(':') {
                        host = buffer;
                        buffer = String::new();
                        state = UrlParsingState::PortState;
                    } else if next_char == None || next_char == Some('/') || next_char == Some('?') || next_char == Some('#') {
                        host = buffer;
                        buffer = String::new();
                        state = UrlParsingState::PathStartState;
                    } else {
                        buffer.push(next_char.unwrap());
                    }
                },

                UrlParsingState::NoSchemeState => {
                    if next_char == Some('#') && base_url.is_some() && !base_url.unwrap().path.is_empty() {
                        let base_url = base_url.unwrap();
                        scheme = base_url.scheme.clone();
                        path = base_url.path.clone();
                        query = base_url.query.clone();
                        fragment = base_url.fragment.clone();
                        state = UrlParsingState::FragmentState;
                    } else if base_url.is_some() && base_url.unwrap().scheme != "file" {
                        pointer = max(pointer - 1, -1);
                        state = UrlParsingState::RelativeState;
                    } else {
                        pointer = max(pointer - 1, -1);
                        state = UrlParsingState::FileState;
                    }
                },

                UrlParsingState::RelativeState =>  {
                    scheme = base_url.unwrap().scheme.clone();
                    if next_char == Some('/') {
                        state = UrlParsingState::RelativeSlashState;
                    } else {
                        let base_url = base_url.unwrap();
                        username = base_url.username.clone();
                        password = base_url.password.clone();
                        host = base_url.host.clone();
                        port = base_url.port.clone();
                        path = base_url.path.clone();
                        query = base_url.query.clone();
                        if next_char == Some('?') {
                            query = String::new();
                            state = UrlParsingState::QueryState;
                        } else if next_char == Some('#') {
                            fragment = String::new();
                            state = UrlParsingState::FragmentState;
                        } else if next_char != None {
                            query = String::new();
                            if !path.is_empty() {
                                path.remove(path.len() - 1);
                            }
                            state = UrlParsingState::PathState;
                            pointer = max(pointer - 1, -1);
                        }
                    }

                },

                UrlParsingState::RelativeSlashState => {
                    if Url::scheme_is_special(&scheme) && next_char == Some('/') {
                        state = UrlParsingState::SpecialAuthorityIgnoreSlashesState;
                    } else if next_char == Some('/') {
                        state = UrlParsingState::AuthorityState;
                    } else {
                        let base_url = base_url.unwrap();
                        username = base_url.username.clone();
                        password = base_url.password.clone();
                        host = base_url.host.clone();
                        port = base_url.port.clone();
                        state = UrlParsingState::PathState;
                        pointer = max(pointer - 1, -1);
                    }
                },

                UrlParsingState::QueryState => {
                    //TODO: we are currently ignoring a lot of percentage encoding here that the spec requires
                    if next_char == Some('#') || next_char == None {
                        query.push_str(&buffer);
                        buffer = String::new();
                        if next_char == Some('#') {
                            state = UrlParsingState::FragmentState;
                        }
                    } else {
                        buffer.push(next_char.unwrap());
                    }
                },

                UrlParsingState::FragmentState => {
                    if next_char != None {
                        fragment.push(next_char.unwrap());
                    }
                },

                UrlParsingState::PortState => todo!(), //TODO: add test and implement

            }

            if pointer >= url_str.len() as i32 {
                //we don't check next_char, because we still need to update it to pointer, but we don't do that first because we still need to
                //increase the pointer, and if it then points to EOF, we still need to do 1 loop...
                break;
            }

            pointer += 1;

            //we sometimes need to make pointer go back, iterators can't do that, so here we just reset the iterator and skip to the correct point
            //when that happens:
            next_char = if pointer == previous_pointer + 1 {
                previous_pointer = pointer;
                url_str_iter.next()
            } else {
                previous_pointer = pointer;
                url_str_iter = url_str.chars().peekable();
                url_str_iter.nth(pointer as usize)
            };
        }

        return Url { scheme, username, password, host, port, path, query, fragment, blob }
    }

    fn parse_host(host_text: &String) -> String {
        //TODO: this should actually implement https://url.spec.whatwg.org/#concept-host-parser
        return host_text.clone();
    }

    pub fn to_string(&self) -> String {
        let mut full_string = String::new();

        full_string.push_str(&self.scheme);
        if self.scheme != "about" {  //TODO: this is a hack, I'm missing something in the URL spec to make this work I think (about: should not have slashes)
            full_string.push_str("://");
        } else {
            full_string.push_str(":");
        }
        full_string.push_str(&self.host);
        if self.scheme != "about" {  //TODO: this is a hack, I'm missing something in the URL spec to make this work I think (about: should not have slashes)
            full_string.push_str("/");
        }
        full_string.push_str(self.path.join("/").as_str());

        if !self.query.is_empty() {
            full_string.push('?');
            full_string.push_str(&self.query);
        }

        return full_string;
    }

    pub fn file_extension(&self) -> Option<String> {
        let last_path_part = self.path.last();
        if last_path_part.is_none() {
            return None;
        }
        let dot_position = last_path_part.unwrap().find('.');
        if dot_position.is_none() {
            return None;
        }
        let extension_start = dot_position.unwrap() + 1;
        return Some(last_path_part.unwrap()[extension_start..].to_lowercase());
    }

    fn scheme_is_special(scheme: &str) -> bool {
        //Special is meant in the definition of https://url.spec.whatwg.org/#is-special
        // the spec also checks for ws and wss, but we are not implementing those
        return scheme == "http" || scheme == "https" || scheme == "ftp" || scheme == "file";
    }
}

