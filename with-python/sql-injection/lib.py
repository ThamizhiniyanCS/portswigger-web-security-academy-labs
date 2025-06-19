import requests
import argparse

# Creating a Session Object
# https://docs.python-requests.org/en/latest/api/#request-sessions
SESSION = requests.Session()


def generate_parser() -> argparse.Namespace:
    parser = argparse.ArgumentParser(prog="Remote Code Execution Via Webshell Upload")
    parser.add_argument(
        "-u", "--lab-url", help="Your lab instance URL is required", required=True
    )

    return parser.parse_args()
