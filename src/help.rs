//! Static help text, captured from the markup the Python version passes to rich.

/// Full help text (without the outer `[not bold white]` wrapper).
pub const HELP: &str = concat!(
    r#"[bold cyan][bold cyan]deew[/bold cyan][/bold cyan] "#,
    env!("CARGO_PKG_VERSION"),
    r#"

[yellow]USAGE[/yellow]: [bold cyan]deew[/bold cyan] [[green]-h[/green]] [[green]-v[/green]] [[green]-i[/green] [[bold color(231)]INPUT[/bold color(231)] ...]] [[green]-ti[/green] [bold color(231)]INDEX[/bold color(231)]] [[green]-o[/green] [bold color(231)]DIRECTORY[/bold color(231)]] [[green]-f[/green] [bold color(231)]FORMAT[/bold color(231)]]
            [[green]-b[/green] [bold color(231)]BITRATE[/bold color(231)]] [[green]-dm[/green] [bold color(231)]CHANNELS[/bold color(231)]] [[green]-d[/green] [bold color(231)]DELAY[/bold color(231)]] [[green]-r[/green] [bold color(231)]DRC[/bold color(231)]] [[green]-dn[/green] [bold color(231)]DIALNORM[/bold color(231)]]
            [[green]-in[/green] [bold color(231)]INSTANCES[/bold color(231)]] [[green]-k[/green]] [[green]-mo[/green]] [[green]-fs[/green]] [[green]-fb[/green]] [[green]-lb[/green]] [[green]-la[/green]] [[green]-np[/green]] [[green]-pl[/green]]
            [[green]-cl[/green]] [[green]-c[/green]] [[green]-gc[/green]]

[yellow]FLAGS[/yellow]:
  [green]-h[/green], [green]--help[/green]                  show this help message.
  [green]-v[/green], [green]--version[/green]               show version.
  [green]-i[/green], [green]--input[/green] [[bold color(231)]INPUT[/bold color(231)] ...]     audio file(s) or folder(s)
  [green]-ti[/green], [green]--track[/green][green]-index[/green] [bold color(231)]INDEX[/bold color(231)]    [underline magenta]default:[/underline magenta] [bold color(231)]0[/bold color(231)]
                              select audio track index of input(s)
  [green]-o[/green], [green]--output[/green] [bold color(231)]DIRECTORY[/bold color(231)]      [underline magenta]default:[/underline magenta] current directory
                              specifies output directory
  [green]-f[/green], [green]--format[/green] [bold color(231)]FORMAT[/bold color(231)]         [underline magenta]options:[/underline magenta] [bold color(231)]dd[/bold color(231)] / [bold color(231)]ddp[/bold color(231)] / [bold color(231)]ac4[/bold color(231)] / [bold color(231)]thd[/bold color(231)]
                              [underline magenta]default:[/underline magenta] [bold color(231)]ddp[/bold color(231)]
  [green]-b[/green], [green]--bitrate[/green] [bold color(231)]BITRATE[/bold color(231)]       [underline magenta]options:[/underline magenta] run [green][green]-lb[/green][/green]/[green][green]--list[/green][green]-bitrates[/green][/green]
                              [underline magenta]default:[/underline magenta] run [green][green]-c[/green][/green]/[green][green]--config[/green][/green]
  [green]-dm[/green], [green]--downmix[/green] [bold color(231)]CHANNELS[/bold color(231)]     [underline magenta]options:[/underline magenta] [bold color(231)]1[/bold color(231)] / [bold color(231)]2[/bold color(231)] / [bold color(231)]6[/bold color(231)]
                              specifies downmix, only works for DD/DDP
                              DD will be automatically downmixed to 5.1 in case of a 7.1 source
  [green]-d[/green], [green]--delay[/green] [bold color(231)]DELAY[/bold color(231)]           [underline magenta]examples:[/underline magenta] [bold color(231)]-5.1ms[/bold color(231)], [bold color(231)]+1,52s[/bold color(231)], [bold color(231)]-24@pal[/bold color(231)], [bold color(231)]+10@24000/1001[/bold color(231)]
                              [underline magenta]default:[/underline magenta] [bold color(231)]0ms[/bold color(231)] or parsed from filename
                              specifies delay as ms, s or frame@FPS
                              FPS can be a number, division or ntsc / pal
                              you have to specify negative values as [bold color(231)]-[/bold color(231)][bold color(231)]d=-0ms[/bold color(231)]
  [green]-r[/green], [green]--drc[/green] [bold color(231)]DRC[/bold color(231)]               [underline magenta]options:[/underline magenta] [bold color(231)]film_light[/bold color(231)] / [bold color(231)]film_standard[/bold color(231)] / [bold color(231)]music_light[/bold color(231)] / [bold color(231)]music_standard[/bold color(231)] / [bold color(231)]speech[/bold color(231)]
                              [underline magenta]default:[/underline magenta] [bold color(231)]music_light[/bold color(231)] (this is the closest to the missing none preset)
                              specifies drc profile
  [green]-dn[/green], [green]--dialnorm[/green] [bold color(231)]DIALNORM[/bold color(231)]    [underline magenta]options:[/underline magenta] between [bold color(231)]-31[/bold color(231)] and [bold color(231)]0[/bold color(231)] (in case of [bold color(231)]0[/bold color(231)] DEE's measurement will be used)
                              [underline magenta]default:[/underline magenta] [bold color(231)]0[/bold color(231)]
                              applied dialnorm value between
  [green]-in[/green], [green]--instances[/green] [bold color(231)]INSTANCES[/bold color(231)]  [underline magenta]examples:[/underline magenta] [bold color(231)]1[/bold color(231)], [bold color(231)]4[/bold color(231)], [bold color(231)]50%[/bold color(231)]
                              [underline magenta]default:[/underline magenta] [bold color(231)]50%[/bold color(231)]
                              specifies how many encodes can run at the same time
                              [bold color(231)]50%[/bold color(231)] means [bold color(231)]4[/bold color(231)] on a cpu with 8 threads
                              one DEE can use 2 threads so [bold color(231)]50%[/bold color(231)] can utilize all threads
                              (this option overrides the config's number)
  [green]-k[/green], [green]--keeptemp[/green]              keep temp files
  [green]-mo[/green], [green]--measure[/green][green]-only[/green]         kills DEE when the dialnorm gets written to the progress bar
                              this option overrides format with ddp
  [green]-fs[/green], [green]--force[/green][green]-standard[/green]       force standard profile for 7.1 DDP encoding (384-1024 kbps)
  [green]-fb[/green], [green]--force[/green][green]-bluray[/green]         force bluray profile for 7.1 DDP encoding (768-1664 kbps)
  [green]-lb[/green], [green]--list[/green][green]-bitrates[/green]        list bitrates that DEE can do for DD and DDP encoding
  [green]-la[/green], [green]--long[/green][green]-argument[/green]        print ffmpeg and DEE arguments for each input
  [green]-np[/green], [green]--no[/green][green]-prompt[/green]            skip confirmation questions, print them as notes and continue
  [green]-pl[/green], [green]--print[/green][green]-logos[/green]          show all logo variants you can set in the config
  [green]-cl[/green], [green]--changelog[/green]            show changelog
  [green]-c[/green], [green]--config[/green]                show config and config location
  [green]-gc[/green], [green]--generate[/green][green]-config[/green]      generate a new config"#
);
