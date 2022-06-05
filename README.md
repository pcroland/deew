<a href="https://github.com/pcroland/deew"><img src="https://img.shields.io/github/v/release/pcroland/deew?logo=data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABAAAAAPCAYAAADtc08vAAACp0lEQVQokW2Tu29UVxDGf3Mfu3d3vWuvszKOke0iGAXHD4IESRqMDCJlqKBIR6SUkdLwD+RBS4ciuaWxKJIIN6AISzTYwhZBAsUyIWDjB8brZX33effeM9G9RoiCkY50Zs7Mp/m+mSPragiBLjhfQm4CeT5gBja32lxf8/WXvX00bCcx3gHk4fte5LcAZgUWQ6WWFT6Jk3YC7c5bcibncBQI7m/qV6u7LOdT4EiMcnBWDPz5WvXSRyKT+0a/Xg258mlK2O3Af4GSMoxNFOWPLwdkab+hY9s+j+UfY8hIQoECwhtVSiJ/+RHTDxoqp/PCCx/+bSoeUKnC9LBsZGy6Zx9ql+UDbQUbzopyp2US/suh0i6BZ8N8qFyu1KBehUoF7q7opG2T+/xj+cGKs30FoxwROGcOAHyUSE2MyxTKhBUgJsQt5ZA3PrvlGotH+vjJGhN5MiLyjSiLEaCaALgoohGJ12zpwmAXn02PyNr4YRkZKAjbe9xwHAqWB8fyMCTKvtGDeSWmoG/vJqLp2eQ8h/6MQ6aYBVvYiN8SChHUVXGFDwOoSdo3b+Na8aETMhC7zlakx8OItSxMWfZ7RRqrkEyYtM3Yyha36w1OBwFPN3ZhfJhvowjfCYW/gxAKLv3xUlgH9ZEqIppoEFiIU21QNx3ulcvQ101vXw9fPFnjR6fVgqANNVgX5WmrkwCkUbx6myYwaEO9x4WsCyYH507w0CjNpVWuOY+33yk/t1fTuVP9Sdcjro0820IbVXZ6c9DjgZfi2Jlxfk+7DN5aYLIZgHMoJfEOkHYYHfL0giv6qyIvsymYPCxXG03oL9I9OsRUJs1oTG/+ESfXX/OoWAB5WVViDUo5vsunmXlWVhkuykXbYub9n9kJ2dksc/35Dj+/qhBGsdgC/wNo0jVGqhwhLwAAAABJRU5ErkJggg=="><a/>
<a href="https://github.com/pcroland/deew/blob/master/LICENSE"><img src="https://img.shields.io/github/license/pcroland/deew?color=red"><a/>
<a href="https://github.com/pcroland/deew/commits/master"><img src="https://img.shields.io/github/last-commit/pcroland/deew?color=blue"><a/>
<a href="https://github.com/pcroland/deew/issues"><img src="https://img.shields.io/github/issues/pcroland/deew?color=blue"><a/>
<br>
<a href="https://github.com/pcroland/deew"><img src="https://img.shields.io/badge/platform-win | linux | osx-blue"><a/>
<a href="https://github.com/pcroland/deew"><img src="https://img.shields.io/badge/python-3.8 | 3.9 | 3.10-blue?logo=Python&logoColor=white"><a/>
<a href="https://github.com/pcroland/deew"><img src="https://img.shields.io/badge/DEE-5.1.0-blue?logoColor=white&logo=data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABAAAAAMCAYAAABr5z2BAAAA9ElEQVQokZXSzypFURQG8N89LsnEv1KGBp5CKU9wBygPIJ5AeYM7MjLAzMBAmRuYyszU3AOgG6GQpZ19OM69Tny1Wu39rVXfWt9qRcQaljCOUbygh3c/MYQJDOMZ9zhvYwEb+hG1n9aAmlaBkfzYwQo2cZUbqnGJdSxjP/eMiYi9+MRiRKjEWXzjpMZ1MnNUVORM1+R1ck6jrNa4qZJrD5irRFrUHW4aahRNJN7w2lTQpCBhBpN/VfBU4w5yTr53a1war0/BPK7zQW1lS0tsYxa7eMDcFxMRhxW7/ovjpOAUj9madMppabe/nHKyOvWkcXu4+AAd1Ju1TsOvFgAAAABJRU5ErkJggg=="><a/>
<hr>

<p align="center"><img width="192" src="logo/logo.svg"><br>Dolby Encoding Engine Wrapper</p>

## DDP encoding has never been so easy!

![img](https://telegra.ph/file/70c800b153b9fe9a88509.gif)
<!---https://i.kek.sh/KjLQCZoQpVx.gif--->

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
- checks if intermediate file is already created so you can encode different formats/bitrates using a single intermediate file, for example:\
  ./deew.py -f dd -b 448 -i input -k\
  ./deew.py -f dd -b 640 -i input -k\
  ./deew.py -f ddp -i input
- works even with video inputs (first audio will be selected)
- fancy terminal output using rich
- versatile delay option that supports ms, s and also frame@fps formats

# Requirements
- Python
- ffmpeg
- ffprobe
- Dolby Encoding Engine

# Installation
```sh
git clone https://github.com/pcroland/deew
cd deew
pip install -r requirements.txt
```
- rename `config.toml.example` to `config.toml` and edit the settings
- install your DEE (if you use WSL use the Win version for better performance)
- place your `license.lic` file next to the DEE binary

You can also grab a standalone exe from here: [https://github.com/pcroland/deew/releases](https://github.com/pcroland/deew/releases)\
*(Doesn't require you to have Python and/or libraries installed)*

# Usage
```
❯ ./deew.py
deew 2.0

USAGE: deew.py [-h] [-v] [-i [INPUT ...]] [-o OUTPUT] [-f FORMAT] [-b BITRATE]
               [-dm DOWNMIX] [-d DELAY] [-drc DRC] [-dn DIALNORM] [-t THREADS]
               [-k] [-mo] [-la] [-pl] [-cl]

FLAGS:
  -h, --help                           show this help message.
  -v, --version                        show version.
  -i [INPUT ...], --input [INPUT ...]  audio file(s) or folder(s)
  -o OUTPUT, --output OUTPUT           output directory
                                       default: current directory
  -f FORMAT, --format FORMAT           dd / ddp / thd
                                       default: ddp
  -b BITRATE, --bitrate BITRATE        defaults:
                                       DD:  1.0: 128 kbps, 2.0: 256 kbps, 5.1: 640 kbps
                                       DDP: 1.0: 128 kbps, 2.0: 256 kbps, 5.1: 1024 kbps, 7.1: 1536 kbps
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
  -la, --long-argument                 print ffmpeg and DEE arguments for each input
  -pl, --printlogos                    show all logo variants you can set in the config
  -cl, --changelog                     show changelog
```
# Examples
`./deew.py -i *thd`\
encode DDP

`./deew.py -b 768 -i *flac`\
encode DDP@768

`./deew,oy -dm 2 -f dd-b 192 *.ec3`\
encode DD@192 with stereo downmixing

`./deew.py -f dd -b 448 -t 4 -i S01`\
encode DD@448 using 4 threads (input is a folder)

`./deew.py -f thd -i *w64`\
encode TrueHD

`./deew.py -f dd -i *dts -k`\
`./deew.py -f ddp -i *dts`\
encode multiple formats/bitrates while creating the temp file only once

# Support
[https://t.me/deew_support](https://t.me/deew_support)\
*(You can ask for help in this group.)*
