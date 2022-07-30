[![builds](https://img.shields.io/github/workflow/status/pcroland/deew/Build%20and%20publish?logo=github&style=flat-square)](https://github.com/pcroland/deew/actions/workflows/build.yaml)
[![github_release](https://img.shields.io/github/v/release/pcroland/deew?logo=github&color=70920c&style=flat-square)](https://github.com/pcroland/deew/releases)
[![pypi_release](https://img.shields.io/pypi/v/deew?label=PyPI&logo=pypi&logoColor=ffffff&color=70920c&style=flat-square)](https://pypi.org/project/deew)
[![license](https://img.shields.io/github/license/pcroland/deew?color=blueviolet&style=flat-square)](https://github.com/pcroland/deew/blob/master/LICENSE)
\
[![telegram](https://img.shields.io/endpoint?color=1d93d2&style=flat-square&url=https://cadoth.net/tgmembercount%3Fchat_id=deew_support%26name=Discussion%2520and%2520Support)](https://t.me/deew_support)
[![commits](https://img.shields.io/github/last-commit/pcroland/deew?color=355ab8&logo=github&style=flat-square)](https://github.com/pcroland/deew/commits/main)
[![issues](https://img.shields.io/github/issues/pcroland/deew?color=355ab8&logo=github&style=flat-square)](https://github.com/pcroland/deew/issues)
\
[![name](https://img.shields.io/badge/platform-win%20%7C%20linux%20%7C%20osx-eeeeee?style=flat-square)](https://github.com/pcroland/deew)
[![name](https://img.shields.io/pypi/pyversions/deew?logo=Python&logoColor=eeeeee&color=eeeeee&style=flat-square)](https://github.com/pcroland/deew)
[![name](https://img.shields.io/badge/DEE-5.1.0--5.2.1-green?logoColor=white&logo=data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABAAAAAMCAYAAABr5z2BAAAA9ElEQVQokZXSzypFURQG8N89LsnEv1KGBp5CKU9wBygPIJ5AeYM7MjLAzMBAmRuYyszU3AOgG6GQpZ19OM69Tny1Wu39rVXfWt9qRcQaljCOUbygh3c/MYQJDOMZ9zhvYwEb+hG1n9aAmlaBkfzYwQo2cZUbqnGJdSxjP/eMiYi9+MRiRKjEWXzjpMZ1MnNUVORM1+R1ck6jrNa4qZJrD5irRFrUHW4aahRNJN7w2lTQpCBhBpN/VfBU4w5yTr53a1war0/BPK7zQW1lS0tsYxa7eMDcFxMRhxW7/ovjpOAUj9madMppabe/nHKyOvWkcXu4+AAd1Ju1TsOvFgAAAABJRU5ErkJggg==&color=eeeeee&style=flat-square)](https://customer.dolby.com/content-creation-and-delivery/dolby-encoding-engine-with-ac-4)
<hr>
<p align="center"><img width="192" src="https://raw.githubusercontent.com/pcroland/deew/main/logo/logo.svg"><br>Dolby Encoding Engine Wrapper</p>


<p align="center"><a href="https://github.com/pcroland/deew/blob/main/README.md">English readme</a>
 • <a href="https://github.com/pcroland/deew/blob/main/README.hu.md">Magyar readme</a></p>

## DDP encode-olás még sosem volt ilyen egyszerű!

![img](https://telegra.ph/file/70c800b153b9fe9a88509.gif)
<!---https://i.kek.sh/KjLQCZoQpVx.gif--->

# Leírás
- kezeli Dolby XML input baromságát a háttérben, rendes CLI felületet adva
- átkonvertálja az inputokat rf64-re, amit már DEE is tud kezelni
  - a bitmélységet, csatornák számát és egyéb infókat a forrásból parse-olja
- minden input fájlhoz generál egy XML fájlt a beállítások alapján
- a script thread poolingot használ batch encodingra (lásd config)
- támogatja a WSL útvonalak konvertálását a DEE Win verziójához (lásd config)
- hibás bitráta megadása esetén kiválasztja a legközelebbi megengedettet
- automatikus mintavételezési ráta konvertálás ffmpeg soxr resamplerét használva nem támogatott mintavételezési ráta esetén
  - dd/dddp esetén a mintavételezési rátát 48000-re konvertálja
  - thd esetén a mintavételezési rátát 48000-re konvertálja, ha a forrásé kisebb mint 72000, fölötte 96000-re
- automatikus csatorna felcserélés 7.1-es forrásoknál (DEE valamiért megcseréli az Ls Rs csatornák az Lrs Rrs-sel)
- automatikus dialnorm beállítás
- automatikusan kompenzálja DEE 256 mintavételezési eltolását (DD és DDP encoding esetén)
- ellenőrzi, hogy az ideiglenes fájl már létezik-e, ezzel lehetővé téve különböző formátumok/bitráták kódolását anélkül, hogy mindegyiknél új ideiglenes fájlt generálnánk, például:\
  `deew -f dd -b 448 -i input -k`\
  `deew -f dd -b 640 -i input -k`\
  `deew -f ddp -i input`
- akár videó inputokkal is működik (az első audió kerül kiválasztásra)
- csicsás terminál kimenet rich-csel
- sokoldalú delay opció, ami támogat ms, s and és frame@fps formát is

# Követelmények
- Python *(nincs rá szükséged, ha a standalone build-et használod)*
- ffmpeg
- ffprobe
- Dolby Encoding Engine

# Dolby Encoding Engine telepítése
- telepítsd fel a DEE-t
  - TrueHD encode-oláshoz csak a Windows verzió használható
  - ha WSL1-et használsz, használd a Windows verziót a jobb teljesítményért
  - ha a Windows verziót használod Linux (és nem WSL) vagy macOS alól, telepítsd fel a `wine-binfmt`-t
- másold a `license.lic` fájlod a DEE binárisod mellé (Windowson `dee.exe`, Linux/maxOS-en `dee`)

# deew telepítése
### a standalone build-et használva (Windows / Linux)
- tölsd le a legfrissebb buildet innen: [https://github.com/pcroland/deew/releases](https://github.com/pcroland/deew/releases)
- futtasd:
```sh
deew
```
- az első futtatáskor készíteni fog egy config fájlt, válaszd ki, hogy melyik elérést szeretnéd használni
*(terminálból futtasd, duplaklikk nem fog működni)*

### Python környezetet használva (Windows / Linux / macOS)
- telepítsd a `python`-t és `pip`-et, ha még nincs fent
- futtasd a következő parancsot: `pip install deew`
- futtasd:
```sh
deew
```
- az első futtatáskor készíteni fog egy config fájlt

# Használat
```
❯ ./deew.py -h
deew 2.3.3

USAGE: deew [-h] [-v] [-i [INPUT ...]] [-o OUTPUT] [-f FORMAT] [-b BITRATE]
            [-dm DOWNMIX] [-d DELAY] [-drc DRC] [-dn DIALNORM] [-t THREADS] [-k]
            [-mo] [-fs] [-fb] [-lb] [-la] [-np] [-pl] [-cl]

FLAGS:
  -h, --help                           show this help message.
  -v, --version                        show version.
  -i [INPUT ...], --input [INPUT ...]  audio file(s) or folder(s)
  -o OUTPUT, --output OUTPUT           output directory
                                       default: current directory
  -f FORMAT, --format FORMAT           dd / ddp / thd
                                       default: ddp
  -b BITRATE, --bitrate BITRATE        defaults: see config
  -dm DOWNMIX, --downmix DOWNMIX       1 / 2 / 6
                                       specifies down/upmix, only works for DD/DDP
                                       DD will be automatically downmixed to 5.1 in case of a 7.1 source
  -d DELAY, --delay DELAY              specifies delay as ms, s or frame@FPS
                                       FPS can be a number, division or ntsc / pal
                                       + / - can also be defined as p / m
                                       examples: -5.1ms, +1,52s, p5s, m5@pal, +10@24000/1001
                                       default: 0ms
  -drc DRC                             film_light / film_standard / music_light / music_standard / speech
                                       drc profile
                                       default: film_light
  -dn DIALNORM, --dialnorm DIALNORM    applied dialnorm value between -31 and 0
                                       0 means auto (DEE's measurement will be used)
                                       default: 0
  -t THREADS, --threads THREADS        number of threads to use, only works for batch encoding,
                                       individial encodes can't be parallelized
                                       default: all threads-1
  -k, --keeptemp                       keep temp files
  -mo, --measure-only                  kills DEE when the dialnorm gets written to the progress bar
                                       this option overwrites format with ddp
  -fs, --force-standard                forces standard profile for 7.1 DDP encoding (384-1024 kbps)
  -fb, --force-bluray                  forces bluray profile for 7.1 DDP encoding (768-1664 kbps)
  -lb, --list-bitrates                 lists bitrates that DEE can do for DD and DDP encoding
  -la, --long-argument                 print ffmpeg and DEE arguments for each input
  -np, --no-prompt                     disables prompt
  -pl, --print-logos                   show all logo variants you can set in the config
  -cl, --changelog                     show changelog
```
# Példák
`deew -i *thd`\
DDP encode-olása

`deew -b 768 -i *flac`\
DDP@768 encode-olása

`deew -dm 2 -f dd -b 192 -i *.ec3`\
DD@192 encode-olása stereo downmixeléssel

`deew -f dd -b 448 -t 4 -i S01`\
DD@448 encode-olása 4 threadet használva (az input egy mappa)

`deew -f thd -i *w64`\
TrueHD encode-olása

`deew -f dd -i *dts -k`\
`deew -f ddp -i *dts`\
több formátum/bitráta kódolása egy ideiglenes fájl készítésével

# Support
[https://t.me/deew_support](https://t.me/deew_support)\
*(Ebben a csoportban tudsz segítséget kérni.)*
