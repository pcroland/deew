# deew 2.0.3:
- proper error handling for missing keys in config

# deew 2.0.2:
- completely disabled xml validation when DEE's platform is not Windows

# deew 2.0.1:
- added `-np`/`--no-prompt`

# deew 2.0.0:
- colorized argparse
- replaced `multiprocessing` with `ThreadPoolExecutor`
    - multiple progress bars for batch encoding
    - dee's percentage remapping is removed (reverted to 1.2.8 behaviour)\
      because the TrueHD encoder works way differently
    - spinning animation for ffmpeg when length can't be parsed
    - removed `TimeRemainingColumn` because it can't be reset,\
      and it only showed the first step's remaining time
- proper error when `ffprobe` fails
- added `-d`/`--delay` option
    - supports ms, s and frame@fps
    - value has to start with `m`/`-` or `p`/`+`
    - fps can be int, float, division or `ntsc`/`pal`
    - examples: -5.1ms, +1,52s, p5s, m5@pal, +10@24000/1001
- added `mono` and `stereo` encoding with a warning prompt
- added `-dn`/`--dialnorm` option with a warning prompt
- added `-dm`/`--downmix` option
- rewrote bitrate and `encoder_mode` selection
- `bluray` `encoder_mode` (for bitrates > 1024) works on Linux/Mac now
- better encoding summary using `rich.table`
    - added `show_summary` option to config
    - shows input/output settings
    - shows if a new version is available
- added `threads` option to config
    - you can overwrite it with -t/--threads.
    - the threads number will be clamped between 1 and `cpu_count() - 2`
- dialnorm appears in the progress bar when the measuring step is done
- added `-mo`/`--measure-only` option
    - kills DEE when the dialnorm value gets written to the progress bar
    - overwrites format with `ddp` if specified
- `ffmpeg` and `DEE` arguments are printed just once with filename placeholders
  - files that already have an intermediate file will be listed
  - you can display the arguments for each file with actual filenames
    with `-la`/`--long-argument`
- standalone exe build
- icon for the project

# deew 1.2.11:
- fixed variable

# deew 1.2.10:
- switched `-af` to `-filter_complex` for resampling, so channel swapping\
  and sample rate conversion works at the same time

# deew 1.2.9:
- disabled `surround_90_degree_phase_shift`
- `DEE`'s measure and encoding step's progress has been remapped,\
  both step goes to 100% now

# deew 1.2.8:
- set `<surround_3db_attenuation>` to `false` in `thd.xml`

# deew 1.2.7:
- prior to this version deew swapped Ls Rs with Lrs Rrs, now it passes\
  `-filter_complex pan=7.1|c0=c0|c1=c1|c2=c2|c3=c3|c4=c6|c5=c7|c6=c4|c7=c5`\
  to 7.1 sources.
- moved ffmpeg args setup out of for loop

# deew 1.2.6:
- added `-cl`/`--changelog` option
- better argparse
- added types and improvements by nyuszika7h
- single input encodes shows a per encode progress bar that indicates both\
  `ffmpeg`'s temp creation and `DEE`'s measuring and encoding step
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
