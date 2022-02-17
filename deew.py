#!/usr/bin/env python3
import argparse
import json
import os
import platform
import random
import shutil
import signal
import subprocess
import sys
import time
from glob import glob
from multiprocessing import Pool, cpu_count
from types import SimpleNamespace

import toml
import xmltodict
from rich import print
from rich.progress import track

signal.signal(signal.SIGINT, signal.SIG_DFL)

parser = argparse.ArgumentParser(add_help=False, formatter_class=argparse.RawTextHelpFormatter)
parser.add_argument('-h', '--help',
                    action='help',
                    default=argparse.SUPPRESS,
                    help='shows this help message.')
parser.add_argument('-v', '--version',
                    action='version',
                    version='deew 1.1',
                    help='shows version.')
parser.add_argument('-i', '--input',
                    nargs='*',
                    default=argparse.SUPPRESS,
                    help='audio file or folder inputs')
parser.add_argument('-f', '--format',
                    type=str,
                    default='ddp',
                    help='dd/ddp/thd (default: ddp)')
parser.add_argument('-b', '--bitrate',
                    type=int,
                    default=0,
                    help='default:\nDD5.1: 640\nDDP5.1: 1024\nDDP7.1: 1536')
parser.add_argument('-m', '--mix',
                    type=int,
                    default=None,
                    help='specify down/upmix (6/8),\nonly works for DDP\ndefault: None')
parser.add_argument('-t', '--threads',
                    type=int,
                    default=cpu_count() - 1,
                    help='number of threads to use, only works for batch encoding,\nindividial encodes can\'t be parallelized\ndefault: all threads-1')
parser.add_argument('-k', '--keeptemp',
                    action='store_true',
                    help='keep temp files')
parser.add_argument('-p', '--progress',
                    action='store_true',
                    help='use progress bar instead of command printing')
args = parser.parse_args()


def printlogo():
    print('''[yellow] ▄▄▄▄▄  ▄▄▄▄▄ ▄▄▄▄▄ ▄▄  ▄▄  ▄▄
 ██  ██ ██▄▄  ██▄▄  ██  ██  ██
 ██  ██ ██    ██    ██  ██  ██
 ▀▀▀▀▀  ▀▀▀▀▀ ▀▀▀▀▀  ▀▀▀▀▀▀▀▀[/yellow]
 [bold color(231)]Dolby Encoding Engine Wrapper[/bold color(231)]
''')


def clamp(inp, low, high):
    return min(max(inp, low), high)


def wpc(p):
    if wsl:
        p = p.split('/')[2:]
        p[0] = p[0].upper() + ':'
        p = '\\'.join(p)
    return p


def openxml(fl):
    with open(fl, 'r') as fl:
        return xmltodict.parse(fl.read())


def savexml(fl, xml):
    with open(fl, 'w') as fl:
        fl.write(xmltodict.unparse(xml, pretty=True, indent='  '))
        fl.close()


def opentoml(fl):
    with open(fl, 'r') as fl:
        return toml.load(fl)


def basename(fl, format_):
    return os.path.basename(os.path.splitext(fl)[0]) + f'.{format_}'


def printexit(text):
    print(text)
    sys.exit(1)


def encode(settings):
    fl = settings[0]
    xml = settings[1]
    channels = settings[2]
    bitdepth = settings[3]
    aformat = args.format.lower()

    xml['job_config']['input']['audio']['wav']['file_name'] = f'\"{basename(fl, "wav")}\"'

    if aformat == 'ddp':
        if channels == '8':
            xml['job_config']['output']['ec3']['file_name'] = f'\"{basename(fl, "eb3")}\"'
        else:
            xml['job_config']['output']['ec3']['file_name'] = f'\"{basename(fl, "ec3")}\"'
    elif aformat == 'dd':
        xml['job_config']['output']['ec3']['file_name'] = f'\"{basename(fl, "ac3")}\"'
        xml['job_config']['output']['ac3'] = xml['job_config']['output']['ec3']
        del xml['job_config']['output']['ec3']
    else:
        xml['job_config']['output']['mlp']['file_name'] = f'\"{basename(fl, "thd")}\"'
    savexml(os.path.join(config['temp_path'], basename(fl, 'xml')), xml)

    if wsl or platform.system() == 'Windows':
        dee_out_path = f'{wpc(config["temp_path"])}\{basename(fl, "xml")}'
    else:
        dee_out_path = f'{config["temp_path"]}/{basename(fl, "xml")}'
    ffmpeg_args = [config['ffmpeg_path'], '-y', '-drc_scale', '0', '-i', fl, '-c:a:0', f'pcm_s{bitdepth}le', '-rf64', 'always', os.path.join(config['temp_path'], basename(fl, 'wav'))]
    dee_args = [config['dee_path'], '-x', dee_out_path]
    ffmpeg_args_print = f'[bold blue]{config["ffmpeg_path"]}[/bold blue] -y -drc_scale [bold color(231)]0[/bold color(231)] -i [bold green]{fl}[/bold green] [not bold white]-c:a[/not bold white]' + f'[not bold white]:0[/not bold white] [bold color(231)]pcm_s{bitdepth}le[/bold color(231)] -rf64 [bold color(231)]always[/bold color(231)] [bold magenta]{os.path.join(config["temp_path"], basename(fl, "wav"))}[/bold magenta]'
    dee_args_print = f'[bold blue]{config["dee_path"]}[/bold blue] -x [bold magenta]{dee_out_path}[/bold magenta]'

    if not args.progress:
        time.sleep(random.uniform(0, 1))
        print(f'{ffmpeg_args_print} && {dee_args_print}', flush=True)
    subprocess.run(ffmpeg_args, stderr=subprocess.DEVNULL, stdout=subprocess.DEVNULL)
    subprocess.run(dee_args, stderr=subprocess.DEVNULL, stdout=subprocess.DEVNULL)

    if not args.keeptemp:
        os.remove(os.path.join(config['temp_path'], basename(fl, 'wav')))
        os.remove(os.path.join(config['temp_path'], basename(fl, 'xml')))

    if aformat == 'thd':
        os.remove(os.path.join(os.getcwd(), basename(fl, 'thd.log')))
        os.remove(os.path.join(os.getcwd(), basename(fl, 'thd.mll')))


def main():
    if len(sys.argv) == 1:
        parser.print_help(sys.stderr)
        sys.exit(1)

    printlogo()

    aformat = args.format.lower()
    bitrate = args.bitrate
    mix = args.mix

    if aformat not in ['dd', 'ddp', 'thd']: printexit('[red]ERROR: [bold yellow]-f[/bold yellow]/[bold yellow]--format[/bold yellow] can only be [bold yellow]dd[/bold yellow], [bold yellow]ddp[/bold yellow] or [bold yellow]thd[/bold yellow].[/red]')
    if aformat != 'ddp' and mix: printexit('[red]ERROR: [bold yellow]-m[/bold yellow]/[bold yellow]--mix[/bold yellow] can only be used for [bold yellow]ddp[/bold yellow] encoding.[/red]')
    if mix and mix not in [6, 8]: printexit('[red]ERROR: [bold yellow]-m[/bold yellow]/[bold yellow]--mix[/bold yellow] can only be [bold yellow]6[/bold yellow] or [bold yellow]8[/bold yellow].[/red]')

    filelist = []
    for f in args.input:
        if not os.path.exists(f):
            printexit(f'[red]ERROR: path [bold yellow]{f}[/bold yellow] does not exist.[/red]')
        if os.path.isdir(f):
            filelist.extend(glob(f + os.path.sep + '*'))
        else:
            filelist.append(f)

    samplerate_list = []
    channels_list = []
    bitdepth_list = []

    for f in filelist:
        probe_args = ['ffprobe', '-v', 'quiet', '-select_streams', 'a:0', '-print_format', 'json', '-show_format', '-show_streams', f]
        output = subprocess.check_output(probe_args)
        audio = json.loads(output)['streams'][0]
        samplerate_list.append(int(audio['sample_rate']))
        channels_list.append(audio['channels'])
        depth = int(audio.get('bits_per_sample', 0))
        if depth == 0: depth = int(audio.get('bits_per_raw_sample', 32))
        bitdepth_list.append(depth)

    if not samplerate_list.count(samplerate_list[0]) == len(samplerate_list):
        printexit(f'[red]ERROR: each input has to have the same sample rate:[/red]\n{samplerate_list}')
    if not channels_list.count(channels_list[0]) == len(channels_list):
        printexit(f'[red]ERROR: each input has to have the same channel count:[/red]\n{channels_list}')
    if not bitdepth_list.count(bitdepth_list[0]) == len(bitdepth_list):
        printexit(f'[red]ERROR: each input has to have the same bit depth:[/red]\n{bitdepth_list}')

    channels = channels_list[0]
    samplerate = samplerate_list[0]
    bitdepth = bitdepth_list[0]
    if bitdepth not in [16, 24, 32]:
        if bitdepth < 16: bitdepth = 16
        elif 16 < bitdepth < 24: bitdepth = 24
        else: bitdepth: 32

    if channels not in [6, 8]: printexit('''[red]ERROR: number of channels can only be [bold yellow]6[/bold yellow] or [bold yellow]8[/bold yellow].
For mono and stereo encoding use [bold blue]qaac[/bold blue] or [bold blue]opus[/bold blue].
For surround tracks with weird channel layouts use [bold blue]Dolby Media Producer[/bold blue] to encode them as is
or use [bold blue]ffmpeg[/bold blue] to remap them ([bold yellow]-ac 6[/bold yellow]/[bold yellow]8[/bold yellow] or [bold yellow]-af "pan=filter"[/bold yellow] for more control) before encoding.[/red]''')
    if aformat in ['dd', 'ddp'] and samplerate != 48000:
        printexit('''[red]ERROR: sample rate for [bold yellow]dd[/bold yellow] and [bold yellow]ddp[/bold yellow] can only be [bold yellow]48000[/bold yellow], use [bold blue]sox[/bold blue] for sample rate conversion:[/red]
[white][bold blue]ffmpeg[/bold blue] -drc_scale [bold color(231)]0[/bold color(231)] -i [bold color(231)]input[/bold color(231)] -v [bold color(231)]quiet[/bold color(231)] -f [bold color(231)]sox[/bold color(231)] - | [bold blue]sox[/bold blue] -p -S -b [bold color(231)]16[/bold color(231)]/[bold color(231)]24[/bold color(231)]/[bold color(231)]32[/bold color(231)] output rate [bold color(231)]48000[/bold color(231)][/white]''')
    if aformat == 'thd' and samplerate not in [48000, 96000]:
        printexit('''[red]ERROR: sample rate for [bold yellow]thd[/bold yellow] can only be [bold yellow]48000[/bold yellow] or [bold yellow]96000[/bold yellow], use [bold blue]sox[/bold blue] for sample rate conversion:[/red]
[white][bold blue]ffmpeg[/bold blue] -drc_scale [bold color(231)]0[/bold color(231)] -i [bold color(231)]input[/bold color(231)] -v [bold color(231)]quiet[/bold color(231)] -f [bold color(231)]sox[/bold color(231)] - | [bold blue]sox[/bold blue] -p -S -b [bold color(231)]16[/bold color(231)]/[bold color(231)]24[/bold color(231)]/[bold color(231)]32[/bold color(231)] output rate [bold color(231)]48000[/bold color(231)]/[bold color(231)]96000[/bold color(231)][/white]''')

    if aformat in ['dd', 'ddp']:
        xmlbase = openxml(os.path.join(script_path, 'xml', 'ddp.xml'))
        xmlbase['job_config']['output']['ec3']['storage']['local']['path'] = f'\"{wpc(os.getcwd())}\"'
        if aformat == 'ddp':
            if (channels == 8 or mix == 8) and mix != 6:
                if bitrate > 1664: printexit('[red]ERROR: bitrate for [bold yellow]7.1 ddp[/bold yellow] can only be [bold yellow]1664[/bold yellow] or lower.[/red]')
                if bitrate == 0: bitrate = '1536'
                xmlbase['job_config']['filter']['audio']['pcm_to_ddp']['encoder_mode'] = 'bluray'
            else:
                if bitrate > 1024: printexit('[red]ERROR: bitrate for [bold yellow]5.1 ddp[/bold yellow] can only be [bold yellow]1024[/bold yellow] or lower.[/red]')
                if bitrate == 0: bitrate = '1024'
                # having downmix 5.1 vs off in case of a 5.1 input results with the same hash
                xmlbase['job_config']['filter']['audio']['pcm_to_ddp']['downmix_config'] = '5.1'
                xmlbase['job_config']['filter']['audio']['pcm_to_ddp']['encoder_mode'] = 'ddp'
        if aformat == 'dd':
            if bitrate > 640: printexit('[red]ERROR: bitrate for [bold yellow]dd[/bold yellow] can only be [bold yellow]640[/bold yellow] or lower.[/red]')
            elif bitrate == 0: bitrate = '640'
            xmlbase['job_config']['filter']['audio']['pcm_to_ddp']['downmix_config'] = '5.1'
            xmlbase['job_config']['filter']['audio']['pcm_to_ddp']['encoder_mode'] = 'dd'
        xmlbase['job_config']['filter']['audio']['pcm_to_ddp']['data_rate'] = bitrate
    elif aformat == 'thd':
        xmlbase = openxml(os.path.join(script_path, 'xml', 'thd.xml'))
        xmlbase['job_config']['output']['mlp']['storage']['local']['path'] = f'\"{wpc(os.getcwd())}\"'

    xmlbase['job_config']['input']['audio']['wav']['storage']['local']['path'] = f'\"{wpc(config["temp_path"])}\"'
    xmlbase['job_config']['misc']['temp_dir']['path'] = f'\"{wpc(config["temp_path"])}\"'

    pformat = '[bold cyan]'
    if aformat == 'dd':
        pformat += f'DD5.1@{bitrate}'
        if channels == 8:
            pformat += ' [not bold white](downmixed from 7.1)[/not bold white]'
    elif aformat == 'ddp':
        pformat += 'DDP'
        if (channels == 8 or mix == 8) and mix != 6:
            pformat += f'7.1@{bitrate}'
            if channels == 6: pformat += ' [not bold white](upmixed from 5.1)[/not bold white]'
        else:
            pformat += f'5.1@{bitrate}'
            if channels == 8: pformat += ' [not bold white](downmixed from 7.1)[/not bold white]'
    else:
        pformat += 'TrueHD.'
        if channels == 6:
            pformat += '5.1'
        else:
            pformat += '7.1'
    pformat += '[/bold cyan]'
    print(f'encoding {pformat}[not bold white]...[/not bold white]')

    threads = clamp(args.threads, 1, cpu_count() - 1)
    pool = Pool(threads)

    settings = []
    for i in range(len(filelist)):
        settings.append([filelist[i], xmlbase, channels, bitdepth])

    if args.progress:
        for audio in track(pool.imap_unordered(encode, settings), total=len(filelist), description='encoding...'):
            pass
    else:
        for audio in pool.imap_unordered(encode, settings):
            pass


script_path = os.path.dirname(__file__)
if not os.path.exists(os.path.join(script_path, 'config.toml')): printexit('[red]ERROR: rename [bold yellow]config.toml.example[/bold yellow] to [bold yellow]config.toml[/bold yellow] and edit the settings.[/red]')
config = opentoml(os.path.join(script_path, 'config.toml'))
if not os.path.exists(str(shutil.which(config['dee_path']))): printexit(f'[red]ERROR: [bold yellow]{config["dee_path"]}[/bold yellow] does not exist.[/red]')
if not os.path.exists(str(shutil.which(config['ffmpeg_path']))): printexit(f'[red]ERROR: [bold yellow]{config["ffmpeg_path"]}[/bold yellow] does not exist.[/red]')
if not os.path.exists(config['temp_path']): printexit(f'[red]ERROR: [bold yellow]{config["temp_path"]}[/bold yellow] does not exist.[/red]')
wsl = True if config['wsl'] else False


if __name__ == "__main__":
    main()
