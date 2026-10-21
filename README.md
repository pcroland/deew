[![builds](https://img.shields.io/github/actions/workflow/status/pcrlnd/deew/build-cli.yaml?logo=github&style=flat-square)](https://github.com/pcrlnd/deew/actions/workflows/build-cli.yaml)
[![github_release](https://img.shields.io/github/v/release/pcrlnd/deew?logo=github&color=70920c&style=flat-square)](https://github.com/pcrlnd/deew/releases)
[![license](https://img.shields.io/github/license/pcrlnd/deew?color=blueviolet&style=flat-square)](https://github.com/pcrlnd/deew/blob/main/LICENSE)
\
[![Telegram](https://telegram-badge.vercel.app/api/telegram-badge?channelId=@deew_supp&style=flat-square&label=Discussion+and+support)](https://t.me/deew_supp)
[![commits](https://img.shields.io/github/last-commit/pcrlnd/deew?color=355ab8&logo=github&style=flat-square)](https://github.com/pcrlnd/deew/commits/main)
[![open_issues](https://img.shields.io/github/issues/pcrlnd/deew?color=718bcd&logo=github&style=flat-square)](https://github.com/pcrlnd/deew/issues)
[![closed_issues](https://img.shields.io/github/issues-closed/pcrlnd/deew?color=253e80&logo=github&style=flat-square)](https://github.com/pcrlnd/deew/issues?q=is%3Aissue+is%3Aclosed)
\
[![name](https://img.shields.io/badge/platform-win%20%7C%20linux%20%7C%20osx-eeeeee?style=flat-square)](https://github.com/pcrlnd/deew)
[![name](https://img.shields.io/badge/DEE-5.1.0--5.2.1-green?logoColor=white&logo=data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABAAAAAMCAYAAABr5z2BAAAA9ElEQVQokZXSzypFURQG8N89LsnEv1KGBp5CKU9wBygPIJ5AeYM7MjLAzMBAmRuYyszU3AOgG6GQpZ19OM69Tny1Wu39rVXfWt9qRcQaljCOUbygh3c/MYQJDOMZ9zhvYwEb+hG1n9aAmlaBkfzYwQo2cZUbqnGJdSxjP/eMiYi9+MRiRKjEWXzjpMZ1MnNUVORM1+R1ck6jrNa4qZJrD5irRFrUHW4aahRNJN7w2lTQpCBhBpN/VfBU4w5yTr53a1war0/BPK7zQW1lS0tsYxa7eMDcFxMRhxW7/ovjpOAUj9madMppabe/nHKyOvWkcXu4+AAd1Ju1TsOvFgAAAABJRU5ErkJggg==&color=eeeeee&style=flat-square)](https://customer.dolby.com/content-creation-and-delivery/dolby-encoding-engine-with-ac-4)
<hr>
<p align="center"><img width="192" src="https://raw.githubusercontent.com/pcrlnd/deew/main/logo/logo.svg"><br>Dolby Encoding Engine Wrapper</p>


<p align="center"><a href="https://github.com/pcrlnd/deew/blob/main/README.md">English readme</a>
 • <a href="https://github.com/pcrlnd/deew/blob/main/README_hu.md">Magyar leírás</a></p>

## DDP encoding has never been so easy!

![img](https://telegra.ph/file/4e75ac457c8f122dfc9a9.gif)
<!---https://i.kek.sh/f2Iv7nZ2ucf.gif--->

# Description
- handles Dolby's XML input fuckery in the background, giving you a proper CLI interface
- converts inputs to RF64 which DEE can use
  - bit depth, number of channels and other infos are parsed from the source
- an XML file will be generated for each input based on the settings
- runs multiple encodes in parallel for batch encoding (see config)
- supports WSL path conversion for the Windows version of DEE (see config)
- in case of an invalid bitrate it will pick the closest allowed one
- automatic sample rate conversion using ffmpeg's soxr resampler in case of an unsupported sample rate
  - for DD/DDP/AC4 unsupported rates will be converted to 48 kHz
  - for TrueHD unsupported rates will be converted to 48 kHz if source sample rate is lower than 72 kHz, otherwise will be converted to 96 kHz
- automatic channel swapping for 7.1 sources (DEE swaps Ls Rs with Lrs Rrs for some reason)
- automatic dialnorm setting
- automatically compensates for DEE's 256 sample delay (DD and DDP encoding)
- checks if intermediate file is already created so you can encode different formats/bitrates using a single intermediate file, for example:\
  `deew -f dd -b 448 -i input -k`\
  `deew -f dd -b 640 -i input -k`\
  `deew -f ddp -i input`
- works even with video inputs (first audio will be selected)
- colored terminal output with live progress bars
- versatile delay option that supports ms, s and also frame@fps formats
- parsing delay from filenames
- optional desktop GUI (deew-gui) and web UI (deew-web) that build the command for you and show the live output

# Requirements
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
### with standalone build (Windows 10-11/Linux):
- grab the latest build (`deew` / `deew.exe`) from: [https://github.com/pcrlnd/deew/releases](https://github.com/pcrlnd/deew/releases)
- on Linux make it executable: `chmod +x deew`
- run with: `deew` (`deew.exe` on Windows)\
*(run the binary from terminal, doubleclicking it won't work)*
- on the first run it will create a config file next to the deew binary
- updating: grab the latest build from: [https://github.com/pcrlnd/deew/releases](https://github.com/pcrlnd/deew/releases)

### build from source (Windows/Linux/macOS):
- install Rust if you don't have it already: [https://rustup.rs](https://rustup.rs)
- run: `cargo install deew`
- run with: `deew`
- on the first run it will create a config file
- updating: run the same command again: `cargo install deew`

*The original Python implementation is kept in the [`python/`](python/) folder for reference.*

# GUI
- download `deew-gui` / `deew-gui.exe` from [https://github.com/pcrlnd/deew/releases](https://github.com/pcrlnd/deew/releases) and put it in the same folder as the CLI binary (it looks for `deew` next to itself, `deew.exe` on Windows)
- on Linux make the binaries executable: `chmod +x deew deew-gui`
- Linux needs the WebKitGTK runtime: `libwebkit2gtk-4.1-0` (Debian/Ubuntu), `webkit2gtk4.1` (Fedora), `webkit2gtk-4.1` (Arch)
- Windows 10/11 already include the needed WebView2 runtime
- the GUI has its own version number, independent from the CLI
- build from source: `cd gui && npm install && npm run tauri build -- --no-bundle` (the binary will be in `gui/target/release/`), for development use `npm run tauri dev`

# Web UI
- `deew-web` is the GUI in your browser, handy for a headless server / NAS that runs DEE
- download `deew-web` / `deew-web.exe` from [https://github.com/pcrlnd/deew/releases](https://github.com/pcrlnd/deew/releases) and put it in the same folder as the CLI binary, on Linux run `chmod +x deew-web`
- run it and open [http://127.0.0.1:8765](http://127.0.0.1:8765), use `--host 0.0.0.0` to make it reachable from other machines and `--port` to change the port
- no WebKit needed, it is a single binary (the Linux one is static)
- login: run `deew-web --hash-password` and paste the result into the `[deew_web]` section of `config.toml` (next to the CLI) as `pass`, set `user` too, leave them empty for no login
- by default it only listens on localhost, anyone who can reach it can run deew and browse the files of the machine, and it speaks plain HTTP, so put it behind a TLS reverse proxy if you expose it to the internet
- build from source: `cd gui && npm install && npm run build && cargo build -p deew-web --release` (the binary will be in `gui/target/release/`)

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
deew 3.5.0

USAGE: deew [-h] [-v] [-i [INPUT ...]] [-ti INDEX] [-o DIRECTORY] [-f FORMAT]
            [-b BITRATE] [-dm CHANNELS] [-d DELAY] [-r DRC] [-dn DIALNORM]
            [-in INSTANCES] [-k] [-mo] [-fs] [-fb] [-lb] [-la] [-np] [-pl]
            [-cl] [-c] [-gc]

FLAGS:
  -h, --help                  show this help message.
  -v, --version               show version.
  -i, --input [INPUT ...]     audio file(s) or folder(s)
  -ti, --track-index INDEX    default: 0
                              select audio track index of input(s)
  -o, --output DIRECTORY      default: current directory
                              specifies output directory
  -f, --format FORMAT         options: dd / ddp / ac4 / thd
                              default: ddp
  -b, --bitrate BITRATE       options: run -lb/--list-bitrates
                              default: run -c/--config
  -dm, --downmix CHANNELS     options: 1 / 2 / 6
                              specifies downmix, only works for DD/DDP
                              DD will be automatically downmixed to 5.1 in case of a 7.1 source
  -d, --delay DELAY           examples: -5.1ms, +1,52s, -24@pal, +10@24000/1001
                              default: 0ms or parsed from filename
                              specifies delay as ms, s or frame@FPS
                              FPS can be a number, division or ntsc / pal
                              you have to specify negative values as -d=-0ms
  -r, --drc DRC               options: film_light / film_standard / music_light / music_standard / speech
                              default: music_light (this is the closest to the missing none preset)
                              specifies drc profile
  -dn, --dialnorm DIALNORM    options: between -31 and 0 (in case of 0 DEE's measurement will be used)
                              default: 0
                              applied dialnorm value between
  -in, --instances INSTANCES  examples: 1, 4, 50%
                              default: 50%
                              specifies how many encodes can run at the same time
                              50% means 4 on a cpu with 8 threads
                              one DEE can use 2 threads so 50% can utilize all threads
                              (this option overrides the config's number)
  -k, --keeptemp              keep temp files
  -mo, --measure-only         kills DEE when the dialnorm gets written to the progress bar
                              this option overrides format with ddp
  -fs, --force-standard       force standard profile for 7.1 DDP encoding (384-1024 kbps)
  -fb, --force-bluray         force bluray profile for 7.1 DDP encoding (768-1664 kbps)
  -lb, --list-bitrates        list bitrates that DEE can do for DD and DDP encoding
  -la, --long-argument        print ffmpeg and DEE arguments for each input
  -np, --no-prompt            skip confirmation questions, print them as notes and continue
  -pl, --print-logos          show all logo variants you can set in the config
  -cl, --changelog            show changelog
  -c, --config                show config and config location
  -gc, --generate-config      generate a new config
```
# Examples
`deew -i *thd`\
encode DDP

`deew -b 768 -i *flac`\
encode DDP@768

`deew -dm 2 -f dd -b 192 -i *.ec3`\
encode DD@192 with stereo downmixing

`deew -f dd -b 448 -in 4 -i S01`\
encode DD@448 using 4 instances (input is a folder)

`deew -f thd -i *w64`\
encode TrueHD

`deew -f dd -i *dts -k`\
`deew -f ddp -i *dts`\
encode multiple formats/bitrates while creating the temp file only once

# Discussion and Support
[https://t.me/deew_support](https://t.me/deew_support)
