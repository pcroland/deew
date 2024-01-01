header_placeholder

## DDP encoding has never been so easy!

![img](https://telegra.ph/file/4e75ac457c8f122dfc9a9.gif)
<!---https://i.kek.sh/f2Iv7nZ2ucf.gif--->

# Description
description_placeholder

# Requirements
- Python *(you don't need it if you use a standalone build of deew)*
- ffmpeg
- ffprobe
- Dolby Encoding Engine

# Dolby Encoding Engine installation
- install [DEE](https://customer.dolby.com/content-creation-and-delivery/dolby-encoding-engine-with-ac-4) (if you use macOS, install [DME](https://customer.dolby.com/content-creation-and-delivery/dolby-media-encoder-with-ac-4))
  - for TrueHD encoding support you need the Windows version
  - if you use WSL1 use the Windows version for better performance
  - if you use the Windows version of DEE under Linux (and not from WSL) / macOS install `wine-binfmt`
- place your `license.lic` file next to the DEE binary (`dee.exe` under Windows, `dee` under Linux/macOS)
- if DEE throws `Failed to load library "...dll".` errors when you run deew install [VisualCppRedist AIO](https://github.com/abbodi1406/vcredist/releases)

# deew installation
### with standalone build (Windows 8-11/Linux):
- grab the latest build from: [https://github.com/pcroland/deew/releases](https://github.com/pcroland/deew/releases)
- run with: `deew`\
*(run the binary from terminal, doubleclicking it won't work)*
- on the first run it will create a config file, choose where you want to keep it
- updating: grab the latest build from: [https://github.com/pcroland/deew/releases](https://github.com/pcroland/deew/releases)

### with Python environment (Windows/Linux/macOS):
- install Python and pip if you don't have it already
- run: `pip install deew`
- run with: `deew`
- on the first run it will create a config file
- updating: `pip install deew --upgrade`

# Setup system PATH variable
If you don't want to use full paths for the binaries in your config or when you use them from CLI, I suggest to setup system PATH variables
### Windows:
- open `cmd.exe` as administrator
- run a `setx /m PATH "%PATH%;[location]"` command for each path that contains binaries\
  *(replace* `[location]` *with the path)*
- for example:
```bat
setx /m PATH "%PATH%;C:\bin\dee"
setx /m PATH "%PATH%;C:\bin\ffmpeg"
```
### Linux/macOS:
- add a `PATH="[location]:$PATH"` line in your `~/.bashrc` or `~/.zshrc` file for each path that contains a binary\
  *(replace* `[location]` *with the path)*
- for example:
```sh
PATH="/usr/local/bin/dee:$PATH"
PATH="/usr/local/bin/ffmpeg:$PATH"
```

# Usage
```
❯ deew -h
help_placeholder
```
# Examples
examples_placeholder

# Discussion and Support
[https://t.me/deew_support](https://t.me/deew_support)
