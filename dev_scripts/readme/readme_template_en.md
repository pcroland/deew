header_placeholder

## DDP encoding has never been so easy!

![img](https://telegra.ph/file/4e75ac457c8f122dfc9a9.gif)
<!---https://i.kek.sh/f2Iv7nZ2ucf.gif--->

# Description
description_placeholder

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
help_placeholder
```
# Examples
examples_placeholder

# Discussion and Support
[https://t.me/deew_support](https://t.me/deew_support)
