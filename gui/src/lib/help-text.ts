// Tooltip texts, taken from the help text of the CLI (src/help.rs).

export const HELP = {
  input: "Audio file(s) or folder(s).",
  trackIndex: "Select audio track index of input(s).\nDefault: 0",
  output:
    "Specifies output directory.\nDefault: current directory (the GUI runs deew from your home folder).",
  format: "Options: dd / ddp / ac4 / thd\nDefault: ddp",
  bitrate:
    "Bitrates DEE can do for the selected format (see Utilities > List bitrates).\nDefault: taken from the config (see Utilities > Show config).",
  downmix:
    "Specifies downmix, only works for DD/DDP.\nDD will be automatically downmixed to 5.1 in case of a 7.1 source.",
  delay:
    "Specifies delay as ms, s or frame@FPS.\nFPS can be a number, division or ntsc / pal.\nExamples: -5.1ms, +1,52s, -24@pal, +10@24000/1001\nDefault: 0ms or parsed from filename.\nNegative values are passed as -d=-0ms automatically.",
  drc: "Specifies drc profile.\nOptions: film_light / film_standard / music_light / music_standard / speech\nDefault: music_light (this is the closest to the missing none preset).",
  dialnorm:
    "Applied dialnorm value, between -31 and 0 (in case of 0 DEE's measurement will be used).\nDefault: 0",
  instances:
    "Specifies how many encodes can run at the same time.\nExamples: 1, 4, 50%\n50% means 4 on a cpu with 8 threads. One DEE can use 2 threads so 50% can utilize all threads.\nThis option overrides the config's number. Default: 50% (see config).",
  keepTemp: "Keep temp files.",
  measureOnly:
    "Kills DEE when the dialnorm gets written to the progress bar.\nThis option overrides format with ddp.",
  forceStandard:
    "Force standard profile for 7.1 DDP encoding (384-1024 kbps).\nExcludes force bluray.",
  forceBluray:
    "Force bluray profile for 7.1 DDP encoding (768-1664 kbps).\nExcludes force standard.",
  longArgument: "Print ffmpeg and DEE arguments for each input.",
  noPrompt: "Skip confirmation questions, print them as notes and continue.",
  info: {
    "-h": "Show the help text of the CLI.",
    "-v": "Show version.",
    "-lb": "List bitrates that DEE can do for DD and DDP encoding.",
    "-pl": "Show all logo variants you can set in the config.",
    "-cl": "Show changelog.",
    "-c": "Show config and config location.",
    "-gc": "Generate a new config next to the deew CLI.",
  },
} as const
