header_placeholder

## DDP encode-olás még sosem volt ilyen egyszerű!

![img](https://telegra.ph/file/4e75ac457c8f122dfc9a9.gif)
<!---https://i.kek.sh/f2Iv7nZ2ucf.gif--->

# Leírás
description_placeholder

# Követelmények
- Python *(nincs rá szükséged, ha a standalone buildet használod)*
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
### standalone buildet használva (Windows 8-11/Linux):
- tölsd le a legfrissebb buildet innen: [https://github.com/pcroland/deew/releases](https://github.com/pcroland/deew/releases)
- futtasd: `deew`\
*(terminálból futtasd, duplaklikk nem fog működni)*
- az első futtatáskor készíteni fog egy config fájlt, válaszd ki, hogy melyik elérést szeretnéd használni
- frissítés: tölsd le a legfrissebb buildet innen: [https://github.com/pcroland/deew/releases](https://github.com/pcroland/deew/releases)

### Python környezetet használva (Windows/Linux/macOS):
- telepítsd a Pythont és pip-et, ha még nincs fent
- futtasd a következő parancsot: `pip install deew`
- futtasd: `deew`
- az első futtatáskor készíteni fog egy config fájlt
- frissítés: `pip install deew --upgrade`

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
