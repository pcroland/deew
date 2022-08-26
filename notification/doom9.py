#!/usr/bin/env python3
import re
import requests
import sys
from hashlib import md5

with open('changelog.txt', 'r') as fl:
    changelog = fl.read().replace('`', '"')
    changelog = f'[code]{changelog}[/code]aa'

password = sys.argv[1]
md5 = md5(password.encode()).hexdigest()

session = requests.Session()

data = {
    'vb_login_username': 'pcroland',
    'vb_login_md5password': md5,
    'vb_login_md5password_utf': md5,
    'do': 'login'
}

session.headers = {'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/104.0.0.0 Safari/537.36'}

session.post('https://forum.doom9.org/login.php?do=login', data=data)

r = session.get('https://forum.doom9.org')
if 'You last visited' in r.text: print('Succesful login.')

token = re.search(r'.+SECURITYTOKEN = "(.+)"', r.text)[1]

# post changelog
session.headers = {
    'Accept': '*/*',
    'Accept-Language': 'hu-HU,hu;q=0.9,en-GB;q=0.8,en;q=0.7,en-US;q=0.6',
    'Connection': 'keep-alive',
    'Content-Type': 'application/x-www-form-urlencoded; charset=UTF-8',
    'DNT': '1',
    'Origin': 'https://forum.doom9.org',
    'Referer': 'https://forum.doom9.org/showthread.php?p=1973688',
    'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/104.0.0.0 Safari/537.36',
    'X-Requested-With': 'XMLHttpRequest',
}

data = {
    'securitytoken': token,
    'ajax': '1',
    'ajax_lastpost': '1661462239',
    'message': changelog,
    'wysiwyg': '0',
    'styleid': '0',
    'signature': '1',
    'fromquickreply': '1',
    's': '',
    'securitytoken': token,
    'do': 'postreply',
    't': '184175',
    'p': 'who cares',
    'specifiedpost': '0',
    'parseurl': '1',
    'loggedinuser': '219960',
}

session.post('https://forum.doom9.org/newreply.php?do=postreply&t=184175', data=data)

# update first post
session.headers = {
    'Accept': '*/*',
    'Accept-Language': 'hu-HU,hu;q=0.9,en-GB;q=0.8,en;q=0.7,en-US;q=0.6',
    'Connection': 'keep-alive',
    'Content-Type': 'application/x-www-form-urlencoded; charset=UTF-8',
    'DNT': '1',
    'Origin': 'https://forum.doom9.org',
    'Referer': 'https://forum.doom9.org/showthread.php?p=1973688',
    'Sec-Fetch-Dest': 'empty',
    'Sec-Fetch-Mode': 'cors',
    'Sec-Fetch-Site': 'same-origin',
    'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/104.0.0.0 Safari/537.36',
    'X-Requested-With': 'XMLHttpRequest',
    'sec-ch-ua': '"Chromium";v="104", " Not A;Brand";v="99", "Google Chrome";v="104"',
    'sec-ch-ua-mobile': '?0',
    'sec-ch-ua-platform': '"Windows"',
}

data = {
    'securitytoken': token,
    'do': 'updatepost',
    'ajax': '1',
    'postid': '1969949',
    'wysiwyg': '0',
    'message': 'testest2222555552',
    'reason': '',
    'postcount': '1',
}

session.post('https://forum.doom9.org/editpost.php?do=updatepost&postid=undefined', data=data)