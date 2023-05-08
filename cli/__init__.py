import argparse
from cli.utils import RParse, CustomHelpFormatter
from cli._version import ProgramInfo

# temp
from deew import main_code_temp


def deew_cli():
    parser = RParse(
        prog=ProgramInfo.prog_name,
        add_help=False,
        formatter_class=lambda prog: CustomHelpFormatter(
            prog, width=78, max_help_position=32
        ),
    )
    parser.add_argument(
        "-h",
        "--help",
        action="help",
        default=argparse.SUPPRESS,
        help="show this help message.",
    )
    parser.add_argument(
        "-v",
        "--version",
        action="version",
        version=f"[bold cyan]{ProgramInfo.prog_name}[/bold cyan] [not bold white]{ProgramInfo.prog_version}[/not bold white]",
        help="show version.",
    )
    parser.add_argument(
        "-i",
        "--input",
        nargs="*",
        default=argparse.SUPPRESS,
        help="audio file(s) or folder(s)",
    )
    parser.add_argument(
        "-ti",
        "--track-index",
        type=int,
        default=0,
        metavar="INDEX",
        help="""[underline magenta]default:[/underline magenta] [bold color(231)]0[/bold color(231)]
    select audio track index of input(s)""",
    )
    parser.add_argument(
        "-o",
        "--output",
        default=None,
        metavar="DIRECTORY",
        help="[underline magenta]default:[/underline magenta] current directory\nspecifies output directory",
    )
    parser.add_argument(
        "-f",
        "--format",
        type=str,
        default="ddp",
        help="""[underline magenta]options:[/underline magenta] [bold color(231)]dd[/bold color(231)] / [bold color(231)]ddp[/bold color(231)] / [bold color(231)]ac4[/bold color(231)] / [bold color(231)]thd[/bold color(231)]
    [underline magenta]default:[/underline magenta] [bold color(231)]ddp[/bold color(231)]""",
    )
    parser.add_argument(
        "-b",
        "--bitrate",
        type=int,
        default=None,
        help="""[underline magenta]options:[/underline magenta] run [green]-lb[/green]/[green]--list-bitrates[/green]
    [underline magenta]default:[/underline magenta] run [green]-c[/green]/[green]--config[/green]""",
    )
    parser.add_argument(
        "-dm",
        "--downmix",
        type=int,
        default=None,
        metavar="CHANNELS",
        help="""[underline magenta]options:[/underline magenta] [bold color(231)]1[/bold color(231)] / [bold color(231)]2[/bold color(231)] / [bold color(231)]6[/bold color(231)]
    specifies downmix, only works for DD/DDP
    DD will be automatically downmixed to 5.1 in case of a 7.1 source""",
    )
    parser.add_argument(
        "-d",
        "--delay",
        type=str,
        default=None,
        help="""[underline magenta]examples:[/underline magenta] [bold color(231)]-5.1ms[/bold color(231)], [bold color(231)]+1,52s[/bold color(231)], \
    [bold color(231)]-24@pal[/bold color(231)], [bold color(231)]+10@24000/1001[/bold color(231)]
    [underline magenta]default:[/underline magenta] [bold color(231)]0ms[/bold color(231)] or parsed from filename
    specifies delay as ms, s or frame@FPS
    FPS can be a number, division or ntsc / pal
    you have to specify negative values as [bold color(231)]-[/bold color(231)][bold color(231)]d=-0ms[/bold color(231)]""",
    )
    parser.add_argument(
        "-r",
        "--drc",
        type=str,
        default="music_light",
        help="""[underline magenta]options:[/underline magenta] [bold color(231)]film_light[/bold color(231)] / [bold color(231)]film_standard[/bold color(231)] / \
    [bold color(231)]music_light[/bold color(231)] / [bold color(231)]music_standard[/bold color(231)] / [bold color(231)]speech[/bold color(231)]
    [underline magenta]default:[/underline magenta] [bold color(231)]music_light[/bold color(231)] (this is the closest to the missing none preset)
    specifies drc profile""",
    )
    parser.add_argument(
        "-dn",
        "--dialnorm",
        type=int,
        default=0,
        help="""[underline magenta]options:[/underline magenta] between [bold color(231)]-31[/bold color(231)] and [bold color(231)]0[/bold color(231)] \
    (in case of [bold color(231)]0[/bold color(231)] DEE\'s measurement will be used)
    [underline magenta]default:[/underline magenta] [bold color(231)]0[/bold color(231)]
    applied dialnorm value between""",
    )
    parser.add_argument(
        "-in",
        "--instances",
        type=str,
        default=None,
        help="""[underline magenta]examples:[/underline magenta] [bold color(231)]1[/bold color(231)], [bold color(231)]4[/bold color(231)], [bold color(231)]50%%[/bold color(231)]
    [underline magenta]default:[/underline magenta] [bold color(231)]50%%[/bold color(231)]
    specifies how many encodes can run at the same time
    [bold color(231)]50%%[/bold color(231)] means [bold color(231)]4[/bold color(231)] on a cpu with 8 threads
    one DEE can use 2 threads so [bold color(231)]50%%[/bold color(231)] can utilize all threads
    (this option overrides the config\'s number)""",
    )
    parser.add_argument("-k", "--keeptemp", action="store_true", help="keep temp files")
    parser.add_argument(
        "-mo",
        "--measure-only",
        action="store_true",
        help="kills DEE when the dialnorm gets written to the progress bar\nthis option overrides format with ddp",
    )
    parser.add_argument(
        "-fs",
        "--force-standard",
        action="store_true",
        help="force standard profile for 7.1 DDP encoding (384-1024 kbps)",
    )
    parser.add_argument(
        "-fb",
        "--force-bluray",
        action="store_true",
        help="force bluray profile for 7.1 DDP encoding (768-1664 kbps)",
    )
    parser.add_argument(
        "-lb",
        "--list-bitrates",
        action="store_true",
        help="list bitrates that DEE can do for DD and DDP encoding",
    )
    parser.add_argument(
        "-la",
        "--long-argument",
        action="store_true",
        help="print ffmpeg and DEE arguments for each input",
    )
    parser.add_argument(
        "-np", "--no-prompt", action="store_true", help="disables prompt"
    )
    parser.add_argument(
        "-pl",
        "--print-logos",
        action="store_true",
        help="show all logo variants you can set in the config",
    )
    parser.add_argument(
        "-cl", "--changelog", action="store_true", help="show changelog"
    )
    parser.add_argument(
        "-c", "--config", action="store_true", help="show config and config location(s)"
    )
    parser.add_argument(
        "-gc", "--generate-config", action="store_true", help="generate a new config"
    )
    args = parser.parse_args()
    
    # temp, we should be handling this with a payload
    main_code_temp(args, parser)