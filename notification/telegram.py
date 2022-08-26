#!/usr/bin/env python3
import requests
import sys

token = sys.argv[1]
to = sys.argv[2]
version = sys.argv[3]

with open('changelog.txt', 'r') as fl:
    changelog = fl.read()
    changelog = changelog.split('\n')[1:]
    changelog = '\n'.join(changelog).replace('`', '"')

r = requests.post(
    url=f'https://api.telegram.org/bot{token}/sendMessage',
    data={
        'chat_id': to,
        'parse_mode': 'html',
        'disable_web_page_preview': 'true',
        'text': f'New release!\n<a href="https://github.com/pcroland/deew/releases/tag/{version}">deew {version}</a>:\n<pre>{changelog}</pre>'
    }
)
r.raise_for_status()
