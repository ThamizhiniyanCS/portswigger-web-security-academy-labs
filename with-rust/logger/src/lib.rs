use text_colorizer::Colorize;

pub fn success(message: &str) {
    println!("{}", format!("[+] {}", message).bright_green());
}

pub fn info(message: &str) {
    println!("{}", format!("[!] {}", message).bright_yellow());
}

pub fn error(message: &str) {
    println!("{}", format!("[-] {}", message).bright_red());
}

pub fn error_return(message: &str) -> String {
    format!("[-] {}", message.bright_red())
}
