#!/usr/bin/env python3
import re
import requests
import subprocess
import sys
from hashlib import md5

h_url = sys.argv[1]

session = requests.session()

captcha = session.get(f'https://{h_url}/login')

print(captcha.text)