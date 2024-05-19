//For now the js console just prints to the terminal. //TODO: show it in the browser itself somehow


pub fn log_js_error(error: &str) {
    println!("[JS console] [ERROR] {}", error);
}


pub fn print(text: &str) {
    println!("[JS console] {}", text);
}
