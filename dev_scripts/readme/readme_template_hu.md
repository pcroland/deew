header_placeholder

## DDP encode-olás még sosem volt ilyen egyszerű!

![img](https://telegra.ph/file/4e75ac457c8f122dfc9a9.gif)
<!---https://i.kek.sh/f2Iv7nZ2ucf.gif--->

# Leírás
description_placeholder

# Követelmények
- ffmpeg
- ffprobe
- Dolby Encoding Engine

# Dolby Encoding Engine telepítése
- telepítsd fel a [DEE](https://customer.dolby.com/content-creation-and-delivery/dolby-encoding-engine-with-ac-4) (ha macOS-t használsz akkor a [DME](https://customer.dolby.com/content-creation-and-delivery/dolby-media-encoder-with-ac-4)-t)
  - TrueHD encode-oláshoz csak a Windows-verzió használható
  - ha WSL1-et használsz, használd a Windows-verziót a jobb teljesítményért
  - ha a Windows-verziót használod Linux (és nem WSL) vagy macOS alól, telepítsd fel a `wine-binfmt`-t
- másold a `license.lic` fájlod a DEE binárisod mellé (Windowson `dee.exe`, Linux/maxOS-en `dee`)
- ha DEE `Failed to load library "...dll".` hibákat dob deew használatakor, telepítsd fel a [VisualCppRedist AIO](https://github.com/abbodi1406/vcredist/releases)-t

# deew telepítése
### standalone buildet használva (Windows 10-11/Linux):
- tölsd le a legfrissebb buildet (`deew` / `deew.exe`) innen: [https://github.com/pcrlnd/deew/releases](https://github.com/pcrlnd/deew/releases)
- Linuxon tedd futtathatóvá: `chmod +x deew`
- futtasd: `deew` (Windowson `deew.exe`)\
*(terminálból futtasd, duplaklikk nem fog működni)*
- az első futtatáskor készíteni fog egy config fájlt a deew binary mellé
- frissítés: tölsd le a legfrissebb buildet innen: [https://github.com/pcrlnd/deew/releases](https://github.com/pcrlnd/deew/releases)

### forrásból buildelve (Windows/Linux/macOS):
- telepítsd a Rustot, ha még nincs fent: [https://rustup.rs](https://rustup.rs)
- futtasd a következő parancsot: `cargo install deew`
- futtasd: `deew`
- az első futtatáskor készíteni fog egy config fájlt
- frissítés: futtasd újra ugyanezt a parancsot: `cargo install deew`

*Az eredeti Python implementáció referenciaként a [`python/`](python/) mappában található.*

# GUI
- tölsd le a `deew-gui` / `deew-gui.exe` fájlt innen: [https://github.com/pcrlnd/deew/releases](https://github.com/pcrlnd/deew/releases), és told a CLI binárisod mellé ugyanabba a mappába (a mellette lévő `deew` fájlt keresi, Windowson `deew.exe`-t)
- Linuxon tedd futtathatóvá a binárisokat: `chmod +x deew deew-gui`
- Linuxon szükség van a WebKitGTK runtime-ra: `libwebkit2gtk-4.1-0` (Debian/Ubuntu), `webkit2gtk4.1` (Fedora), `webkit2gtk-4.1` (Arch)
- Windows 10/11 már tartalmazza a szükséges WebView2 runtime-ot
- a GUI saját verziószámot használ, a CLI-tól függetlenül
- forrásból buildelve: `cd gui && npm install && npm run tauri build -- --no-bundle` (a binárist a `gui/target/release/` mappában találod), fejlesztéshez használd: `npm run tauri dev`

# Web UI
- a `deew-web` a GUI a böngésződben, jól jön egy headless szerveren / NAS-on, ahol a DEE fut
- tölsd le a `deew-web` / `deew-web.exe` fájlt innen: [https://github.com/pcrlnd/deew/releases](https://github.com/pcrlnd/deew/releases), és told a CLI binárisod mellé ugyanabba a mappába, Linuxon futtasd: `chmod +x deew-web`
- indítsd el, és nyisd meg: [http://127.0.0.1:8765](http://127.0.0.1:8765), a `--host 0.0.0.0`-val más gépekről is elérhető lesz, a `--port`-tal pedig a portot változtathatod
- nem kell hozzá WebKit, egyetlen binárisról van szó (a Linuxos statikus)
- bejelentkezés: futtasd a `deew-web --hash-password` parancsot, és másold az eredményt a `config.toml` (a CLI mellett) `[deew_web]` szekciójának `pass` mezőjébe, a `user` mezőt is töltsd ki, üresen hagyva nincs bejelentkezés
- alapból csak a localhoston figyel, bárki, aki eléri, futtathat deew-t és böngészheti a gép fájljait, és sima HTTP-t használ, ezért ha az internetre kiteszed, tegyél elé TLS-t tudó reverse proxyt
- forrásból buildelve: `cd gui && npm install && npm run build && cargo build -p deew-web --release` (a binárist a `gui/target/release/` mappában találod)

# Rendszer PATH változók beállítása
Ha nem szeretnéd a teljes elérési utat használni a binárisokhoz a configban, vagy amikor CLI-ből használod őket, javaslom a rendszer PATH változók beállítását
### Windows:
- nyisd meg `cmd.exe`-t adminként
- futtas egy `setx /m PATH "%PATH%;[location]"` parancsot minden mappával, amiben binary van\
  *(a* `[location]`*-t cseréld le az elérési útra)*
- például:
```bat
setx /m PATH "%PATH%;C:\bin\dee"
setx /m PATH "%PATH%;C:\bin\ffmpeg"
```
### Linux/macOS:
- adj hozzá egy `PATH="[location]:$PATH"` sort a `~/.bashrc` vagy `~/.zshrc` fájlodhoz, minden mappával, amiben binary van\
  *(a* `[location]`*-t cseréld le az elérési útra)*
- például:
```sh
PATH="/usr/local/bin/dee:$PATH"
PATH="/usr/local/bin/ffmpeg:$PATH"
```

# Használat
```
❯ deew -h
help_placeholder
```
# Példák
examples_placeholder

# Beszélgetés és support
[https://t.me/deew_support](https://t.me/deew_support)
