#!/usr/bin/env python3
import argparse
import json
import os
import platform
import shutil
import signal
import subprocess
import sys
from copy import copy
from glob import glob
from multiprocessing import Pool, cpu_count

import toml
import xmltodict
from rich import print
from rich.progress import track

from logos import logos

signal.signal(signal.SIGINT, signal.SIG_DFL)

parser = argparse.ArgumentParser(
    add_help=False, formatter_class=lambda prog: argparse.RawTextHelpFormatter(prog, max_help_position=40)
)
parser.add_argument('-h', '--help',
                    action='help',
                    default=argparse.SUPPRESS,
                    help='shows this help message.')
parser.add_argument('-v', '--version',
                    action='version',
                    version='deew 1.2',
                    help='shows version.')
parser.add_argument('-i', '--input',
                    nargs='*',
                    default=argparse.SUPPRESS,
                    help='audio file(s) or folder(s)')
parser.add_argument('-o', '--output',
                    default=None,
                    help='output directory\ndefault: current directory')
parser.add_argument('-f', '--format',
                    type=str,
                    default='ddp',
                    help='dd/ddp/thd\ndefault: ddp')
parser.add_argument('-b', '--bitrate',
                    type=int,
                    default=0,
                    help='defaults:\nDD5.1: 640\nDDP5.1: 1024\nDDP7.1: 1536')
parser.add_argument('-m', '--mix',
                    type=int,
                    default=None,
                    help='6/8\nspecify down/upmix, only works for DDP')
parser.add_argument('-drc',
                    type=str,
                    default='film_light',
                    help='film_light/film_standard/music_light/music_standard/speech\ndrc profile\ndefault: film_light')
parser.add_argument('-t', '--threads',
                    type=int,
                    default=cpu_count() - 1,
                    help='number of threads to use, only works for batch encoding,\nindividial encodes can\'t be parallelized\ndefault: all threads-1')
parser.add_argument('-k', '--keeptemp',
                    action='store_true',
                    help='keep temp files')
parser.add_argument('-pl', '--printlogos',
                    action='store_true',
                    help='show all logo variants you can set in the config')
args = parser.parse_args()


def printlogos():
    for i in range(len(logos)):
        print(f'logo {i + 1}:\n{logos[i]}')
    sys.exit(1)


def clamp(inp, low, high):
    return min(max(inp, low), high)


def findclosestallowed(value, allowed_values):
    return min(allowed_values, key=lambda list_value : abs(list_value - value))


def wpc(p):
    if wsl:
        if not p.startswith('/mnt/'):
            printexit(f'[red]ERROR: WSL path conversion doesn\'t work with [bold yellow]{p}[/bold yellow].[/red]')
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


def createdir(out):
    try:
        if not os.path.exists(out):
            os.makedirs(out)
    except OSError:
        printexit(f'[red]ERROR: Failed to create [bold yellow]{out}[/bold yellow].[/red]')


def encode(settings):
    fl = settings[0]
    output = settings[1]
    ffmpeg_args = settings[2]
    dee_args = settings[3]
    intermediate_exists = settings[4]

    if not intermediate_exists:
        subprocess.run(ffmpeg_args, stderr=subprocess.DEVNULL, stdout=subprocess.DEVNULL)
    subprocess.run(dee_args, stderr=subprocess.DEVNULL, stdout=subprocess.DEVNULL)

    if not args.keeptemp:
        os.remove(os.path.join(config['temp_path'], basename(fl, 'wav')))
        os.remove(os.path.join(config['temp_path'], basename(fl, 'xml')))

    if args.format.lower() == 'thd':
        os.remove(os.path.join(output, basename(fl, 'thd.log')))
        os.remove(os.path.join(output, basename(fl, 'thd.mll')))


def main():
    if len(sys.argv) == 1:
        parser.print_help(sys.stderr)
        sys.exit(1)

    if args.printlogos: printlogos()
    if 0 < config['logo'] < len(logos) + 1: print(logos[config['logo'] - 1])

    aformat = args.format.lower()
    bitrate = args.bitrate
    mix = args.mix

    if aformat not in ['dd', 'ddp', 'thd']: printexit('[red]ERROR: [bold yellow]-f[/bold yellow]/[bold yellow]--format[/bold yellow] can only be [bold yellow]dd[/bold yellow], [bold yellow]ddp[/bold yellow] or [bold yellow]thd[/bold yellow].[/red]')
    if aformat != 'ddp' and mix: printexit('[red]ERROR: [bold yellow]-m[/bold yellow]/[bold yellow]--mix[/bold yellow] can only be used for [bold yellow]ddp[/bold yellow] encoding.[/red]')
    if mix and mix not in [6, 8]: printexit('[red]ERROR: [bold yellow]-m[/bold yellow]/[bold yellow]--mix[/bold yellow] can only be [bold yellow]6[/bold yellow] or [bold yellow]8[/bold yellow].[/red]')
    if args.drc not in ['film_light', 'film_standard', 'music_light', 'music_standard', 'speech']: printexit('[red]ERROR: allowed DRC values: [bold yellow]film_light[/bold yellow], [bold yellow]film_standard[/bold yellow], [bold yellow]music_light[/bold yellow], [bold yellow]music_standard[/bold yellow], [bold yellow]speech[/bold yellow].[/red]')
    if platform.system == 'Linux' and not wsl and aformat == 'thd': printexit('[red]Linux version of DEE does not support TrueHD encoding. set wsl to true in config and use Windows version.[/red]')

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
        probe_args = [config["ffprobe_path"], '-v', 'quiet', '-select_streams', 'a:0', '-print_format', 'json', '-show_format', '-show_streams', f]
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

    if args.output:
        createdir(os.path.abspath(args.output))
        output = os.path.abspath(args.output)
    else:
        output = os.getcwd()

    if aformat in ['dd', 'ddp']:
        xmlbase = openxml(os.path.join(script_path, 'xml', 'ddp.xml'))
        xmlbase['job_config']['output']['ec3']['storage']['local']['path'] = f'\"{wpc(output)}\"'
        if aformat == 'ddp':
            if (channels == 8 or mix == 8) and mix != 6:
                if bitrate == 0: bitrate = 1536
                bitrate = findclosestallowed(bitrate, [768, 1024, 1280, 1536, 1664])
                xmlbase['job_config']['filter']['audio']['pcm_to_ddp']['encoder_mode'] = 'bluray'
            else:
                if bitrate == 0: bitrate = 1024
                bitrate = findclosestallowed(bitrate, [192, 200, 208, 216, 224, 232, 240, 248, 256, 272, 288, 304, 320, 336, 352, 368, 384, 400, 448, 512, 576, 640, 704, 768, 832, 896, 960, 1008, 1024])
                # having downmix 5.1 vs off in case of a 5.1 input results with the same hash
                xmlbase['job_config']['filter']['audio']['pcm_to_ddp']['downmix_config'] = '5.1'
                xmlbase['job_config']['filter']['audio']['pcm_to_ddp']['encoder_mode'] = 'ddp'
        if aformat == 'dd':
            if bitrate == 0: bitrate = 640
            bitrate = findclosestallowed(bitrate, [224, 256, 320, 384, 448, 512, 576, 640])
            xmlbase['job_config']['filter']['audio']['pcm_to_ddp']['downmix_config'] = '5.1'
            xmlbase['job_config']['filter']['audio']['pcm_to_ddp']['encoder_mode'] = 'dd'
        xmlbase['job_config']['filter']['audio']['pcm_to_ddp']['data_rate'] = bitrate
        xmlbase['job_config']['filter']['audio']['pcm_to_ddp']['drc']['line_mode_drc_profile'] = args.drc
        xmlbase['job_config']['filter']['audio']['pcm_to_ddp']['drc']['rf_mode_drc_profile'] = args.drc
    elif aformat == 'thd':
        xmlbase = openxml(os.path.join(script_path, 'xml', 'thd.xml'))
        xmlbase['job_config']['output']['mlp']['storage']['local']['path'] = f'\"{wpc(output)}\"'
        xmlbase['job_config']['filter']['audio']['encode_to_dthd']['atmos_presentation']['drc_profile'] = args.drc
        xmlbase['job_config']['filter']['audio']['encode_to_dthd']['presentation_8ch']['drc_profile'] = args.drc
        xmlbase['job_config']['filter']['audio']['encode_to_dthd']['presentation_6ch']['drc_profile'] = args.drc

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
    print(f'encoding {pformat}[not bold white]...[/not bold white]\n')


    if wsl or platform.system() == 'Windows':
        dee_xml_input_base = f'{wpc(config["temp_path"])}\\'
    else:
        dee_xml_input_base = f'{config["temp_path"]}/'

    settings = []

    threads = clamp(args.threads, 1, cpu_count() - 1)
    pool = Pool(threads)

    if len(filelist) > 1:
        print(f'[bold color(231)]Running the following commands for the encodes ([cyan]{min(len(filelist), threads)}[/cyan] at a time):[/bold color(231)]')
    else:
        print('[bold color(231)]Running the following commands for the encode:[/bold color(231)]')

    for i in range(len(filelist)):
        dee_xml_input = f'{dee_xml_input_base}{basename(filelist[i], "xml")}' 
        if aformat in ['dd', 'ddp'] and samplerate != 48000:
            bitdepth = 32
            resample_args = ['-af', 'aresample=resampler=soxr', '-ar', '48000', '-precision', '28', '-cutoff', '1', '-dither_scale', '0']
            resample_args_print = '-af [bold color(231)]aresample=resampler=soxr[/bold color(231)] -ar [bold color(231)]48000[/bold color(231)] -precision [bold color(231)]28[/bold color(231)] -cutoff [bold color(231)]1[/bold color(231)] -dither_scale [bold color(231)]0[/bold color(231)] '
        elif aformat == 'thd' and samplerate not in [48000, 96000]:
            bitdepth = 32
            if samplerate < 72000:
                resample_value = '48000'
            else:
                resample_value = '96000'
            resample_args = ['-af', 'aresample=resampler=soxr', '-ar', resample_value, '-precision', '28', '-cutoff', '1', '-dither_scale', '0']
            resample_args_print = f'-af [bold color(231)]aresample=resampler=soxr[/bold color(231)] -ar [bold color(231)]{resample_value}[/bold color(231)] -precision [bold color(231)]28[/bold color(231)] -cutoff [bold color(231)]1[/bold color(231)] -dither_scale [bold color(231)]0[/bold color(231)] '
        else:
            resample_args = []
            resample_args_print = ''

        ffmpeg_args = [config['ffmpeg_path'], '-y', '-drc_scale', '0', '-i', filelist[i], '-c:a:0', f'pcm_s{bitdepth}le', *(resample_args),'-rf64', 'always', os.path.join(config['temp_path'], basename(filelist[i], 'wav'))]
        ffmpeg_args_print = f'[bold blue]ffmpeg[/bold blue] -y -drc_scale [bold color(231)]0[/bold color(231)] -i [bold green]{filelist[i]}[/bold green] [not bold white]-c:a[/not bold white]' + f'[not bold white]:0[/not bold white] [bold color(231)]pcm_s{bitdepth}le[/bold color(231)] {resample_args_print}-rf64 [bold color(231)]always[/bold color(231)] [bold magenta]{os.path.join(config["temp_path"], basename(filelist[i], "wav"))}[/bold magenta]'
        dee_args = [config['dee_path'], '-x', dee_xml_input]
        dee_args_print = f'[bold blue]dee[/bold blue] -x [bold magenta]{dee_xml_input}[/bold magenta]'

        intermediate_exists = False
        if os.path.exists(os.path.join(config['temp_path'], basename(filelist[i], 'wav'))):
            intermediate_exists = True
            print(f'[green]Found intermediate file, skipping creating one with ffmpeg[/green] && {dee_args_print}')
        else:
            print(f'{ffmpeg_args_print} && {dee_args_print}')

        xml = copy(xmlbase)
        xml['job_config']['input']['audio']['wav']['file_name'] = f'\"{basename(filelist[i], "wav")}\"'
        if aformat == 'ddp':
            if channels == 8:
                xml['job_config']['output']['ec3']['file_name'] = f'\"{basename(filelist[i], "eb3")}\"'
            else:
                xml['job_config']['output']['ec3']['file_name'] = f'\"{basename(filelist[i], "ec3")}\"'
        elif aformat == 'dd':
            xml['job_config']['output']['ec3']['file_name'] = f'\"{basename(filelist[i], "ac3")}\"'
            xml['job_config']['output']['ac3'] = xml['job_config']['output']['ec3']
            del xml['job_config']['output']['ec3']
        else:
            xml['job_config']['output']['mlp']['file_name'] = f'\"{basename(filelist[i], "thd")}\"'
        savexml(os.path.join(config['temp_path'], basename(filelist[i], 'xml')), xml)

        settings.append([filelist[i], output, ffmpeg_args, dee_args, intermediate_exists])

    for audio in track(pool.imap_unordered(encode, settings), total=len(filelist), description='encoding...'):
        pass


script_path = os.path.dirname(__file__)

if os.path.exists(os.path.join(script_path, 'config.toml')):
    config = opentoml(os.path.join(script_path, 'config.toml'))
elif platform.system() == 'Linux' and os.path.exists(os.path.join(os.path.expanduser('~'), '.config', 'deew', 'config.toml')):
    config = opentoml(os.path.join(os.path.expanduser('~'), '.config', 'deew', 'config.toml'))
else:
    printexit('''[red]ERROR: rename [bold yellow]config.toml.example[/bold yellow] to [bold yellow]config.toml[/bold yellow] and edit the settings.
Config has to be next to the script or it can also be at [bold yellow]~/.config/deew/config.toml[/bold yellow] on Linux.[/red]''')

if not config['temp_path']:
    config['temp_path'] = os.path.join(script_path, 'temp')
config['temp_path'] = os.path.abspath(config['temp_path'])
createdir(config['temp_path'])

if not os.path.exists(str(shutil.which(config['dee_path']))): printexit(f'[red]ERROR: [bold yellow]{config["dee_path"]}[/bold yellow] does not exist.[/red]')
if not os.path.exists(str(shutil.which(config['ffmpeg_path']))): printexit(f'[red]ERROR: [bold yellow]{config["ffmpeg_path"]}[/bold yellow] does not exist.[/red]')
if not os.path.exists(str(shutil.which(config['ffprobe_path']))): printexit(f'[red]ERROR: [bold yellow]{config["ffprobe_path"]}[/bold yellow] does not exist.[/red]')

wsl = True if config['wsl'] else False


if __name__ == "__main__":
    main()
