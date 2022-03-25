# deew 1.2.6:
- added `-cl`/`--changelog` option
- better argparse
- added types and improvements by nyuszika7h
- single input encodes shows a per encode progress bar that indicates both\
  `ffmpeg`'s temp creation and `DEE`'s measuring and encoding step.
- fixed dd multithreading

# deew 1.2:
- better temp folder config:
    - empty: next to the script
    - relative path: from your current directory
    - you can also use fullpath
    - in any case will be created automatically if it doesn't exist already
- added check for intermediate file existence, now you can do:\
  `./deew.py -f dd -b 448 -i input -k`\
  `./deew.py -f dd -b 640 -i input -k`\
  `./deew.py -f ddp -i input`\
  and the intermediate file will only be created once (and removed after the\
  last encode)
- added automatic sample rate conversion with `ffmpeg`'s `soxr` resampler
    - for dd/ddp unsupported rates will be converted to 48000
    - for thd unsupported rates will be converted to 48000 if source sample\
      rate is lower than 72000, otherwise will be converted to 96000
    - command: `-af aresample=resampler=soxr -ar 48000/96000`\
      `-precision 28 -cutoff 1 -dither_scale 0`
- added alternative config path (`~/.config/deew/config.toml`) to Linux
- removed progress option, commands are printed at the same time as the\
  progress bar appears
- added drc option (`film_light`, `film_standard`, `music_light`,\
  `music_standard`, `speech`)
- you can now specify invalid bitrates and it will pick the closest\
  allowed one (`-f dd -b 635` -> 640k)
- added error for TrueHD encoding with Linux version of `DEE`
- all xml files are generated before encoding
- added back logos and logo config
- fixed intermediate file's bit depth to 32 bit if there's resampling involved

# deew 1.1:
- added sample rate, channel count and bit depth parsing for better error\
  handling and reducing temp file sizes (needs `ffprobe`)
    - spits out error for unsupported sample rates and channel counts
    - suggests using qaac or opus for mono and stereo
    - suggests using DMP or ffmpeg's `-ac 6`/`8` and `-af "pan=filter"` for\
      weird surround layouts
    - suggests using `sox` for sample rate conversion in case of unsupported\
      sample rates
    - in a run every input has to have the same channel count, sample rate\
      and bit depth (because of batch encoding)
    - grabs bits_per_sample value, if that doesn't exist or if it's 0 grabs\
      bits_per_raw_sample and if there's no bit depth 32 will be used
    - any bit depth that is not 16, 24 or 32 will be rounded up to the\
      nearest value
- changed channel option to mix
    - only works with DDP
    - DD will be encoded as 5.1 (using `DEE`'s downmix in case of a 7.1 source)
    - up/downmixing is reported at start
- removed dialnorm option
    - automatic measurement is better than relying on the input's metadata
    - `DEE` always measures the dialnorm, so there's no speed gain of setting\
      it manually
    - `deew` with Linux binaries of `DEE` should work now
- removed logo configuration and option and changed the smallest logo to an\
  even smaller one
- changed config.toml to config.toml.example so it throws an error that it\
  should be renamed and edited (to avoid user confusion)
- added temp path existence check (to avoid user confusion)
- removed magic dependency and added a wsl option to the config instead
- added `-drc_scale 0` in the `ffmpeg` arg list just in case someone encodes\
  from a DD/DDP source for whatever reason
- small XML option tweaks by cnzqy1
