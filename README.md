```sh
 ▄▄▄▄▄▄▄   ▄▄▄▄▄▄▄  ▄▄▄▄▄▄▄  ▄▄▄  ▄▄▄  ▄▄▄
 ███  ███  ███      ███      ███  ███  ███
 ███  ███  ███      ███      ███  ███  ███
 ███  ███  ███▄▄▄   ███▄▄▄   ███  ███  ███
 █▓█  ███  █▓█      █▓█      █▓█  █▓█  █▓█
 █▒█  █▓█  █▒█      █▒█      █▒█  █▒█  █▒█
 █░█  █░█  █░█      █░█      █░█  █░█  █░█
 ▀▀▀▀▀▀▀   ▀▀▀▀▀▀▀  ▀▀▀▀▀▀▀   ▀▀▀▀▀▀▀▀▀▀▀
       Dolby Encoding Engine Wrapper
```
## DDP encoding has never been so easy!

![img](https://i.kek.sh/6RSDNILEvbb.gif)

# Description
This wrapper handles Dolby's XML input fuckery in the background,\
giving you a proper CLI interface. The wrapper converts the input\
files to rf64 which DEE can understand. An XML file will be generated\
for each input file based on the settings. The tool utilizes thread\
pooling for batch encoding (all threads-1 by default). Supports WSL\
path conversion for the Win version of DEE. (see `config.toml`)

# Installation
```sh
git clone https://github.com/pcroland/deew
cd deew
pip install -r requirements.txt
```
* edit config.toml
* install your DEE (if you use WSL use the Win version for better performance)
* place your `license.lic` file next to the DEE binary

# Usage
```ruby
./deew.py
usage: deew.py [-h] [-v] [-i [INPUT ...]] [-f FORMAT] [-b BITRATE]
[-c CHANNELS] [-d DIALNORM] [-t THREADS] [-k] [-p] [--printlogos]

optional arguments:
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
  -c CHANNELS, --channels CHANNELS
                        number of channels in the input file (automatically downmixes to 5.1 when encoding DD from 7.1 input).
                        default: 6
  -d DIALNORM, --dialnorm DIALNORM
                        default: -31
  -t THREADS, --threads THREADS
                        number of threads to use.
                        default: all threads-1
  -k, --keeptemp        keep temp files
  -p, --progress        use progress bar instead of command printing
  --printlogos          show all logo variants you can set in the config
```
# Examples
`./deew.py -i *w64`\
encode 5.1 DDP@1024

`./deew.py -b 768 -i *flac`\
encode 5.1 DDP@768

`./deew.py -c 8 -i *thd`\
encode 7.1 DDP@1536

`./deew.py -f dd -b 448 -d -27 -t 4 -i S01`\
encode 5.1 DD@448 with -27 dialnorm using 4 threads

`./deew.py -f thd -i *w64`\
encode 5.1 TrueHD
