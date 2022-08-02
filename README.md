



# Additions of this fork
- process isolated date spoofing to bypass expired license (use -spoof, currently only works on Linux, requires libfaketime)


Below is the same Readme, with changes to include -spoof usage


---
[![builds](https://img.shields.io/github/workflow/status/pcroland/deew/Build%20and%20publish?logo=github&style=flat-square)](https://github.com/pcroland/deew/actions/workflows/build.yaml)
[![github_release](https://img.shields.io/github/v/release/pcroland/deew?logo=github&color=70920c&style=flat-square)](https://github.com/pcroland/deew/releases)
[![pypi_release](https://img.shields.io/pypi/v/deew?label=PyPI&logo=pypi&logoColor=ffffff&color=70920c&style=flat-square)](https://pypi.org/project/deew)
[![pypi_downloads](https://img.shields.io/pypi/dm/deew?color=70920c&logo=pypi&logoColor=white&style=flat-square)](https://pypi.org/project/deew)
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

## DDP encoding has never been so easy!

![img](https://telegra.ph/file/efd2a1d3519bdf87fca03.gif)
<!---https://i.kek.sh/Mk3qQ0QGWUj.gif--->


# Description
- handles Dolby's XML input fuckery in the background, giving you a proper CLI interface
- converts inputs to rf64 which DEE can use
  - bit depth, number of channels and other infos are parsed from the source
- an XML file will be generated for each input based on the settings
- the script utilizes thread pooling for batch encoding (see config)
- supports WSL path conversion for the Win version of DEE (see config)
- in case of an invalid bitrate it will pick the closest allowed one
- automatic sample rate conversion using ffmpeg's soxr resampler in case of an unsupported sample rate
  - for dd/ddp unsupported rates will be converted to 48000
  - for thd unsupported rates will be converted to 48000 if source sample rate is lower than 72000, otherwise will be converted to 96000
- automatic channel swapping for 7.1 sources (DEE swaps Ls Rs with Lrs Rrs for some reason)
- automatic dialnorm setting
- automatically compensates for DEE's 256 sample delay (DD and DDP encoding)
- checks if intermediate file is already created so you can encode different formats/bitrates using a single intermediate file, for example:\
  `deew -f dd -b 448 -i input -k`\
  `deew -f dd -b 640 -i input -k`\
  `deew -f ddp -i input`
- works even with video inputs (first audio will be selected)
- fancy terminal output using rich
- versatile delay option that supports ms, s and also frame@fps formats

# Requirements
- Python
- ffmpeg
- ffprobe
- Dolby Encoding Engine
- libfaketime (Linux only) for licence date spoofing

# Dolby Encoding Engine installation
- install your [DEE](https://customer.dolby.com/content-creation-and-delivery/dolby-encoding-engine-with-ac-4)
  - for TrueHD encoding support you need the Windows version
  - if you use WSL1 use the Windows version for better performance
  - if you use the Windows version of DEE under Linux (and not from WSL) / macOS install `wine-binfmt`
- place your `license.lic` file next to the DEE binary (`dee.exe` under Windows, `dee` under Linux/macOS)

# deew installation with standalone build (Windows/Linux)
- grab the latest build from: [https://github.com/pcroland/deew/releases](https://github.com/pcroland/deew/releases)
- run with: `deew`\
*(run the binary from terminal, doubleclicking it won't work)*
- on the first run it will create a config file, choose where you want to keep it
- updating: grab the latest build from: [https://github.com/pcroland/deew/releases](https://github.com/pcroland/deew/releases)

# deew installation with Python environment (Windows/Linux/macOS)
- install Python and pip if you don't have it already
- run: `pip install deew`
- run with: `deew`
- on the first run it will create a config file
- updating: `pip install deew --upgrade`

# Usage
```
❯ deew -h
deew 2.5.0-spoof-edition

USAGE: deew [-h] [-v] [-i [INPUT ...]] [-o OUTPUT] [-f FORMAT] [-b BITRATE]
            [-dm DOWNMIX] [-d DELAY] [-drc DRC] [-dn DIALNORM] [-t THREADS] [-k]
            [-mo] [-fs] [-fb] [-lb] [-la] [-np] [-pl] [-cl] [-c] [-gc] [-spoof]

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
                                       specifies downmix, only works for DD/DDP
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
  -fs, --force-standard                force standard profile for 7.1 DDP encoding (384-1024 kbps)
  -fb, --force-bluray                  force bluray profile for 7.1 DDP encoding (768-1664 kbps)
  -lb, --list-bitrates                 list bitrates that DEE can do for DD and DDP encoding
  -la, --long-argument                 print ffmpeg and DEE arguments for each input
  -np, --no-prompt                     disables prompt
  -pl, --print-logos                   show all logo variants you can set in the config
  -cl, --changelog                     show changelog
  -c, --config                         show config and config location(s)
  -gc, --generate-config               generate a new config
  -spoof, --spoof-datetime             automatically spoof date to bypass expired license
```
# Examples
`deew -i *thd`\
encode DDP

`deew -b 768 -i *flac`\
encode DDP@768

`deew -dm 2 -f dd -b 192 -i *.ec3`\
encode DD@192 with stereo downmixing

`deew -f dd -b 448 -t 4 -i S01`\
encode DD@448 using 4 threads (input is a folder)

`deew -f thd -i *w64`\
encode TrueHD

`deew -f dd -i *dts -k`\
`deew -f ddp -i *dts`\
encode multiple formats/bitrates while creating the temp file only once

# Discussion and Support
[https://t.me/deew_support](https://t.me/deew_support)
