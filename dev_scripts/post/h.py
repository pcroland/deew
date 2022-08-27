#!/usr/bin/env python3
import requests
import sys

h_url = sys.argv[1]
h_password = sys.argv[2]
h_totp = sys.argv[3]

session = requests.session()

session.get(f'https://{h_url}/login')
