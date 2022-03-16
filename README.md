```sh
 ▄▄▄▄▄  ▄▄▄▄▄ ▄▄▄▄▄ ▄▄  ▄▄  ▄▄
 ██  ██ ██▄▄  ██▄▄  ██  ██  ██
 ██  ██ ██    ██    ██  ██  ██
 ▀▀▀▀▀  ▀▀▀▀▀ ▀▀▀▀▀  ▀▀▀▀▀▀▀▀
 Dolby Encoding Engine Wrapper
```
## DDP encoding has never been so easy!

![img](https://i.kek.sh/PUI0356sddC.gif)

# Description
- handles Dolby's XML input fuckery in the background, giving you a proper CLI interface
- converts inputs to rf64 which DEE can use
  - bit depth is parsed from source
- an XML file will be generated for each input based on the settings
- the script utilizes thread pooling for batch encoding (all threads-1 by default)
- supports WSL path conversion for the Win version of DEE (see config)
- in case of an invalid bitrate it will pick the closest valid one
- automatic sample rate conversion using ffmpeg's soxr resampler in case of an unsupported sample rate
  - for dd/ddp unsupported rates will be converted to 48000
  - for thd unsupported rates will be converted to 48000 if source sample rate is lower than 72000, otherwise will be converted to 96000
- automatic dialnorm setting
- checks if intermediate file is already created so you can encode different formats/bitrates using a single intermediate file, for example:\
  ./deew.py -f dd -b 448 -i input -k\
  ./deew.py -f dd -b 640 -i input -k\
  ./deew.py -f ddp -i input
- works even with video inputs (first audio will be selected)

# Requirements
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

# Usage
```ruby
❯ ./deew.py --help
usage: deew.py [-h] [-v] [-i [INPUT ...]] [-o OUTPUT] [-f FORMAT] [-b BITRATE] [-m MIX] [-drc DRC] [-t THREADS] [-k] [-pl]

options:
  -h, --help                           shows this help message.
  -v, --version                        shows version.
  -i [INPUT ...], --input [INPUT ...]  audio file(s) or folder(s)
  -o OUTPUT, --output OUTPUT           output directory
                                       default: current directory
  -f FORMAT, --format FORMAT           dd/ddp/thd
                                       default: ddp
  -b BITRATE, --bitrate BITRATE        defaults:
                                       DD5.1: 640
                                       DDP5.1: 1024
                                       DDP7.1: 1536
  -m MIX, --mix MIX                    6/8
                                       specify down/upmix, only works for DDP
                                       DD will be automatically downmixed to 5.1 in case of a 7.1 source
  -drc DRC                             film_light/film_standard/music_light/music_standard/speech
                                       drc profile
                                       default: film_light
  -t THREADS, --threads THREADS        number of threads to use, only works for batch encoding,
                                       individial encodes can't be parallelized
                                       default: all threads-1
  -k, --keeptemp                       keep temp files
  -pl, --printlogos                    show all logo variants you can set in the config
```
# Examples
`./deew.py -i *thd`\
encode DDP

`./deew.py -b 768 -i *flac`\
encode DDP with 768kbps

`./deew.py -m 8 -i *dts`\
encode DDP with 7.1 upmixing

`./deew.py -f dd -b 448 -t 4 -i S01`\
encode DD with 448kbps using 4 threads (input is a folder)

`./deew.py -f thd -i *w64`\
encode TrueHD

`./deew.py -f dd -i *dts -k`\
`./deew.py -f ddp -i *dts`\
encode multiple formats/bitrates while creating the temp file just once

# Todo
- pyinstaller standalone exe
- multiple progress bars (currently encoding progress is only shown for a single input, for multiple inputs the progress bar counts the encoded files)
