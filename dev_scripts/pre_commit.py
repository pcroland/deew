#!/usr/bin/env python3
import re
import subprocess

with open('changelog.md') as fl:
    version = fl.read().split('\n')[0]
    version = re.match(r'# deew ([0-9\.]+):', version)[1]

subprocess.run(['poetry', 'version', version])

with open('deew/__main__.py', 'r+') as fl:
    main = fl.read()
    main = re.sub(r'prog_version = \'[0-9\.]+\'', f'prog_version = \'{version}\'', main, count=1)
    fl.seek(0)
    fl.truncate()
    fl.write(main)

help_ = subprocess.run(['python', '-m', 'deew'], capture_output=True, encoding='utf-8').stdout.rstrip('\n')

with open('dev_scripts/readme/readme_template_en.md', encoding='utf-8') as fl:
    readme_en = fl.read()

with open('dev_scripts/readme/readme_template_hu.md', encoding='utf-8') as fl:
    readme_hu = fl.read()

with open('dev_scripts/readme/header.md', encoding='utf-8') as fl:
    header = fl.read()

with open('dev_scripts/readme/description_en.md', encoding='utf-8') as fl:
    description_en = fl.read()

with open('dev_scripts/readme/description_hu.md', encoding='utf-8') as fl:
    description_hu = fl.read()

with open('dev_scripts/readme/examples_en.md', encoding='utf-8') as fl:
    examples_en = fl.read()

with open('dev_scripts/readme/examples_hu.md', encoding='utf-8') as fl:
    examples_hu = fl.read()

readme_en = re.sub('header_placeholder', header, readme_en)
readme_hu = re.sub('header_placeholder', header, readme_hu)
readme_en = re.sub('description_placeholder', description_en, readme_en)
readme_hu = re.sub('description_placeholder', description_hu, readme_hu)
readme_en = re.sub('help_placeholder', help_, readme_en)
readme_hu = re.sub('help_placeholder', help_, readme_hu)
readme_en = re.sub('examples_placeholder', examples_en, readme_en)
readme_hu = re.sub('examples_placeholder', examples_hu, readme_hu)

with open('README.md', 'w') as fl:
    fl.write(readme_en)

with open('README_hu.md', 'w') as fl:
    fl.write(readme_hu)

with open('dev_scripts/readme/help.txt', 'w') as fl:
    fl.write(help_)
