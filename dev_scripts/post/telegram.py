#!/usr/bin/env python3
import requests
import sys

def main():
    token = sys.argv[1]
    to = sys.argv[2]
    version = sys.argv[3]

    with open('changelog.md', 'r') as fl:
        changelog = fl.read()
        changelog = changelog.split('\n\n# deew')[0].split('\n')[1:]
        changelog = '\n'.join(changelog).replace('`', '"').replace('\\\n', '\n')

    print('Sending notification...')
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


if __name__ == '__main__':
    main()
