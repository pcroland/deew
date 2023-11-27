- handles Dolby's XML input fuckery in the background, giving you a proper CLI interface
- converts inputs to RF64 which DEE can use
  - bit depth, number of channels and other infos are parsed from the source
- an XML file will be generated for each input based on the settings
- the script utilizes thread pooling for batch encoding (see config)
- supports WSL path conversion for the Windows version of DEE (see config)
- in case of an invalid bitrate it will pick the closest allowed one
- automatic sample rate conversion using ffmpeg's soxr resampler in case of an unsupported sample rate
  - for DD/DDP/AC4 unsupported rates will be converted to 48 kHz
  - for TrueHD unsupported rates will be converted to 48 kHz if source sample rate is lower than 72 kHz, otherwise will be converted to 96 kHz
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
- parsing delay from filenames