import requests
import argparse


def generate_parser() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        prog="Portswigger Labs - Server Side Attacks - Authentication"
    )
    parser.add_argument(
        "-u", "--lab-url", help="Your lab instance URL is required", required=True
    )

    return parser.parse_args()
