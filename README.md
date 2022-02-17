```sh
 ▄▄▄▄▄  ▄▄▄▄▄ ▄▄▄▄▄ ▄▄  ▄▄  ▄▄
 ██  ██ ██▄▄  ██▄▄  ██  ██  ██
 ██  ██ ██    ██    ██  ██  ██
 ▀▀▀▀▀  ▀▀▀▀▀ ▀▀▀▀▀  ▀▀▀▀▀▀▀▀
 Dolby Encoding Engine Wrapper
```
## DDP encoding has never been so easy!

![img](https://i.kek.sh/gToEgEcaGFw.gif)

# Description
This wrapper handles Dolby's XML input fuckery in the background, giving you a proper CLI interface. The wrapper converts the input files to rf64 which DEE can understand. An XML file will be generated for each input file based on the settings. The tool utilizes thread pooling for batch encoding (all threads-1 by default). Supports WSL path conversion for the Win version of DEE. (see `config.toml`)

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
* rename `config.toml.example` to `config.toml` and edit the settings
* install your DEE (if you use WSL use the Win version for better performance)
* place your `license.lic` file next to the DEE binary

# Usage
```ruby
./deew.py
usage: deew.py [-h] [-v] [-i [INPUT ...]] [-f FORMAT] [-b BITRATE] [-m MIX] [-t THREADS] [-k] [-p]

options:
  -h, --help            shows this help message.
  -v, --version         shows version.
  -i [INPUT ...], --input [INPUT ...]
                        audio file or folder inputs
  -f FORMAT, --format FORMAT
                        dd/ddp/thd (default: ddp)
  -b BITRATE, --bitrate BITRATE
                        default:
                        DD5.1: 640
                        DDP5.1: 1024
                        DDP7.1: 1536
  -m MIX, --mix MIX     specify down/upmix (6/8),
                        only works for DDP
                        default: None
  -t THREADS, --threads THREADS
                        number of threads to use, only works for batch encoding,
                        individial encodes can't be parallelized
                        default: all threads-1
  -k, --keeptemp        keep temp files
  -p, --progress        use progress bar instead of command printing
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
