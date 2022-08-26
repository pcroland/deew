#!/usr/bin/env python3
import re
import subprocess

_help = subprocess.run(['python', '-m', 'deew'], capture_output=True, encoding='utf-8').stdout.rstrip('\n')

with open('dev_scripts/readme/readme_template_en.md', encoding='utf-8') as fl:
    readme_en = fl.read()

with open('dev_scripts/readme/readme_template_hu.md', encoding='utf-8') as fl:
    readme_hu = fl.read()

with open('dev_scripts/readme/header.md', encoding='utf-8') as fl:
    header = fl.read()

with open('dev_scripts/readme/description_en.txt', encoding='utf-8') as fl:
    description_en = fl.read()

with open('dev_scripts/readme/description_hu.txt', encoding='utf-8') as fl:
    description_hu = fl.read()

with open('dev_scripts/readme/examples_en.txt', encoding='utf-8') as fl:
    examples_en = fl.read()

with open('dev_scripts/readme/examples_hu.txt', encoding='utf-8') as fl:
    examples_hu = fl.read()

readme_en = re.sub('header_placeholder', header, readme_en)
readme_hu = re.sub('header_placeholder', header, readme_hu)
readme_en = re.sub('description_placeholder', description_en, readme_en)
readme_hu = re.sub('description_placeholder', description_hu, readme_hu)
readme_en = re.sub('help_placeholder', _help, readme_en)
readme_hu = re.sub('help_placeholder', _help, readme_hu)
readme_en = re.sub('examples_placeholder', examples_en, readme_en)
readme_hu = re.sub('examples_placeholder', examples_hu, readme_hu)

with open('README.md', 'w') as fl:
    fl.write(readme_en)

with open('README_hu.md', 'w') as fl:
    fl.write(readme_hu)
