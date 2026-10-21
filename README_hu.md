[![builds](https://img.shields.io/github/actions/workflow/status/pcrlnd/deew/build-cli.yaml?logo=github&style=flat-square)](https://github.com/pcrlnd/deew/actions/workflows/build-cli.yaml)
[![github_release](https://img.shields.io/github/v/release/pcrlnd/deew?logo=github&color=70920c&style=flat-square)](https://github.com/pcrlnd/deew/releases)
[![license](https://img.shields.io/github/license/pcrlnd/deew?color=blueviolet&style=flat-square)](https://github.com/pcrlnd/deew/blob/main/LICENSE)
\
[![telegram](https://img.shields.io/endpoint?label=Discussion%20%26%20support&style=flat-square&url=https%3A%2F%2Fmogyo.ro%2Fquart-apis%2Ftgmembercount%3Fchat_id%3Ddeew_support)](https://t.me/deew_support)
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

## DDP encode-olás még sosem volt ilyen egyszerű!

![img](https://telegra.ph/file/4e75ac457c8f122dfc9a9.gif)
<!---https://i.kek.sh/f2Iv7nZ2ucf.gif--->

# Leírás
- kezeli a Dolby XML input baromságait a háttérben, rendes CLI felületet adva
- átkonvertálja az inputokat RF64-re, amit már DEE is tud kezelni
  - a bitmélységet, csatornák számát és egyéb infókat a forrásból parse-olja
- minden input fájlhoz generál egy XML fájlt a beállítások alapján
- több kódolást párhuzamosan futtat batch encodingnál (lásd config)
- támogatja a WSL útvonalak konvertálását a DEE Windows verziójához (lásd config)
- hibás bitráta megadása esetén kiválasztja a legközelebbi megengedettet
- automatikus mintavételezésiráta-konvertálás ffmpeg soxr resamplerét használva nem támogatott mintavételezési ráta esetén
  - DD/DDP/AC4 esetén a mintavételezési rátát 48 kHz-re konvertálja
  - TrueHD esetén a mintavételezési rátát 48 kHz-re konvertálja, ha a forrásé kisebb mint 72 kHz, fölötte 96 kHz-re
- automatikus csatornafelcserélés 7.1-es forrásoknál (DEE valamiért megcseréli az Ls, Rs csatornákat az Lrs, Rrs-sel)
- automatikus dialnorm beállítás
- automatikusan kompenzálja a DEE 256 mintavételezés eltolását (DD és DDP encoding esetén)
- ellenőrzi, hogy az ideiglenes fájl létezik-e már, lehetővé téve különböző formátumok/bitráták kódolásását egyetlen ideiglenes fájl készítésével, például:\
  `deew -f dd -b 448 -i input -k`\
  `deew -f dd -b 640 -i input -k`\
  `deew -f ddp -i input`
- akár videó inputokkal is működik (az első audió kerül kiválasztásra)
- színes terminálkimenet élő folyamatjelző sávokkal
- sokoldalú delay opció, ami támogat ms, s and és frame@fps formát is
- delay parse-olása fájlnévből
- opcionális asztali GUI (deew-gui) és webes felület (deew-web), amik összeállítják neked a parancsot, és megjelenítik az élő kimenetet

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
# Példák
`deew -i *thd`\
DDP encode-olása

`deew -b 768 -i *flac`\
DDP@768 encode-olása

`deew -dm 2 -f dd -b 192 -i *.ec3`\
DD@192 encode-olása stereo downmixeléssel

`deew -f dd -b 448 -in 4 -i S01`\
DD@448 encode-olása 4 instance-et használva (az input egy mappa)

`deew -f thd -i *w64`\
TrueHD encode-olása

`deew -f dd -i *dts -k`\
`deew -f ddp -i *dts`\
több formátum/bitráta kódolása egy ideiglenes fájl készítésével

# Beszélgetés és support
[https://t.me/deew_support](https://t.me/deew_support)
