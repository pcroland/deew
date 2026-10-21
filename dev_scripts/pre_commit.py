#!/usr/bin/env python3
import html
import os
import re
import subprocess
import sys

FEATURES_START = '<!-- features:start -->'
FEATURES_END = '<!-- features:end -->'
CHECK_ICON = '<svg class="icon" aria-hidden="true"><use href="#i-circle-check" /></svg>'


def _inline_html(text):
    """Escape text and turn `code` spans and backslash line breaks into HTML."""
    out = []
    for i, line in enumerate(re.split(r'\\\n', text)):
        if i:
            out.append('<br />')
        for j, part in enumerate(re.split(r'`([^`]*)`', line.strip())):
            part = html.escape(part, quote=False)
            out.append(f'<code>{part}</code>' if j % 2 else part)
    return ''.join(out)


def _parse_features(markdown):
    """Return [(text, [child_text, ...]), ...] from a two-level markdown bullet list."""
    items = []
    last = None  # the list (text holder) currently being continued
    for line in markdown.splitlines():
        if line.startswith('- '):
            items.append([line[2:], []])
            last = (items[-1], None)
        elif re.match(r'^\s+- ', line):
            items[-1][1].append(line.strip()[2:])
            last = (items[-1], len(items[-1][1]) - 1)
        elif line.strip() and last:
            item, child = last
            extra = line.strip()
            if child is None:
                item[0] += '\n' + extra
            else:
                item[1][child] += '\n' + extra
    return items


def render_features(markdown):
    cards = []
    for text, children in _parse_features(markdown):
        card = ['          <li class="feature">', f'            {CHECK_ICON}', '            <div class="feature-body">',
                f'              <p>{_inline_html(text)}</p>']
        if children:
            card.append('              <ul class="feature-sub">')
            card += [f'                <li>{_inline_html(c)}</li>' for c in children]
            card.append('              </ul>')
        card += ['            </div>', '          </li>']
        cards.append('\n'.join(card))
    return '\n'.join(cards)


def update_website_features():
    """Regenerate the feature cards in website/index.html from description_en.md."""
    with open('dev_scripts/readme/description_en.md', encoding='utf-8') as fl:
        markdown = fl.read()
    with open('website/index.html', encoding='utf-8', newline='') as fl:
        page = fl.read()
    block = f'{FEATURES_START}\n{render_features(markdown)}\n          {FEATURES_END}'
    page, n = re.subn(
        re.escape(FEATURES_START) + r'.*?' + re.escape(FEATURES_END),
        lambda m: block,
        page,
        count=1,
        flags=re.S,
    )
    if n != 1:
        raise SystemExit('could not find the features markers in website/index.html')
    with open('website/index.html', 'w', encoding='utf-8', newline='') as fl:
        fl.write(page)


if '--features-only' in sys.argv[1:]:
    update_website_features()
    sys.exit(0)

with open('changelog.md') as fl:
    version = fl.read().split('\n')[0]
    version = re.match(r'# deew ([0-9\.]+):', version)[1]

with open('Cargo.toml', encoding='utf-8') as fl:
    cargo = fl.read()

# bump the first `version = "..."` line inside the [package] table
cargo, n = re.subn(
    r'(^\[package\][^\n]*\n(?:(?!^\[).*\n)*?version\s*=\s*")[^"]*(")',
    lambda m: f'{m[1]}{version}{m[2]}',
    cargo,
    count=1,
    flags=re.M,
)
if n != 1:
    raise SystemExit('could not find the version line in the [package] table of Cargo.toml')

with open('Cargo.toml', 'w', encoding='utf-8', newline='') as fl:
    fl.write(cargo)

# running without arguments prints the help to stdout and exits with 1, this also refreshes Cargo.lock
help_ = subprocess.run(
    ['cargo', 'run', '--quiet'],
    capture_output=True,
    encoding='utf-8',
    env={**os.environ, 'NO_COLOR': '1'},
).stdout.rstrip('\n')

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

update_website_features()
