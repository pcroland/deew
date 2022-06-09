#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
import os
import platform
import re
import shutil
import signal
import subprocess
import sys
import time
from concurrent.futures import ThreadPoolExecutor
from copy import deepcopy
from datetime import timedelta
from glob import glob
from multiprocessing import cpu_count
from typing import Any, NoReturn

import requests
import toml
import xmltodict
from packaging import version
from rich import print
from rich.progress import BarColumn, Progress, TaskID
from rich.prompt import Confirm
from rich.table import Table

from bitrates import allowed_bitrates
from logos import logos
from messages import error_messages
from xml_base import xml_dd_ddp_base, xml_thd_base
from version import prog_version

prog_name = 'deew'

col_base = 'not bold white'
col_usage = 'yellow'
col_option = 'green'
col_caps = 'color(231)'
col_prog = 'cyan'

class RParse(argparse.ArgumentParser):
    def _print_message(self, message, file=None):
        if message:
            if message.startswith('usage'):
                message = f'[{col_prog}]{prog_name}[/{col_prog}] {prog_version}\n\n{message}'
                message = re.sub(r'(-[a-z]+\s*|\[)([A-Z]+)(?=]|,|\s\s|\s\.)', r'\1[{}]\2[/{}]'.format(col_caps, col_caps), message)
                message = re.sub(r'((-|--)[a-z]+)', r'[{}]\1[/{}]'.format(col_option, col_option), message)
                message = message.replace('usage', f'[{col_usage}]USAGE[/{col_usage}]')
                message = message.replace('options', f'[{col_usage}]FLAGS[/{col_usage}]')
                message = message.replace(self.prog, f'[{col_prog}]{self.prog}[/{col_prog}]')
            message = f'[{col_base}]{message.strip()}[/{col_base}]'
            print(message)


parser = RParse(
    add_help=False,
    formatter_class=lambda prog: argparse.RawTextHelpFormatter(prog, width=80, max_help_position=40)
)
parser.add_argument('-h', '--help',
                    action='help',
                    default=argparse.SUPPRESS,
                    help='show this help message.')
parser.add_argument('-v', '--version',
                    action='version',
                    version=f'[{col_prog}]{prog_name}[/{col_prog}] [not bold white]{prog_version}[/not bold white]',
                    help='show version.')
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
                    help='dd / ddp / thd\ndefault: ddp')
parser.add_argument('-b', '--bitrate',
                    type=int,
                    default=None,
                    help='defaults:\nDD:  1.0: 128 kbps, 2.0: 256 kbps, 5.1: 640 kbps\nDDP: 1.0: 128 kbps, 2.0: 256 kbps, 5.1: 1024 kbps, 7.1: 1536 kbps')
parser.add_argument('-dm', '--downmix',
                    type=int,
                    default=None,
                    help='1 / 2 / 6\nspecifies down/upmix, only works for DD/DDP\nDD will be automatically downmixed to 5.1 in case of a 7.1 source')
parser.add_argument('-d', '--delay',
                    type=str,
                    default='+0ms',
                    help='specifies delay as ms, s or frame@FPS\nFPS can be a number, division or ntsc / pal\n+ / - can also be defined as p / m\nexamples: -5.1ms, +1,52s, p5s, m5@pal, +10@24000/1001\ndefault: 0ms')
parser.add_argument('-drc',
                    type=str,
                    default='film_light',
                    help='film_light / film_standard / music_light / music_standard / speech\ndrc profile\ndefault: film_light')
parser.add_argument('-dn', '--dialnorm',
                    type=int,
                    default=0,
                    help='applied dialnorm value between -31 and 0\n0 means auto (DEE\'s measurement will be used)\ndefault: 0')
parser.add_argument('-t', '--threads',
                    type=int,
                    default=None,
                    help='number of threads to use, only works for batch encoding,\nindividial encodes can\'t be parallelized\ndefault: all threads-1')
parser.add_argument('-k', '--keeptemp',
                    action='store_true',
                    help='keep temp files')
parser.add_argument('-mo', '--measure-only',
                    action='store_true',
                    help='kills DEE when the dialnorm gets written to the progress bar\nthis option overwrites format with ddp')
parser.add_argument('-la', '--long-argument',
                    action='store_true',
                    help='print ffmpeg and DEE arguments for each input')
parser.add_argument('-np', '--no-prompt',
                    action='store_true',
                    help='disables prompt')
parser.add_argument('-pl', '--printlogos',
                    action='store_true',
                    help='show all logo variants you can set in the config')
parser.add_argument('-cl', '--changelog',
                    action='store_true',
                    help='show changelog')
args = parser.parse_args()


def print_changelog() -> None:
    try:
        changelog = requests.get('https://raw.githubusercontent.com/pcroland/deew/main/changelog.md').text.split('\n')
    except Exception:
        print_exit('changelog')

    for line in changelog:
        line = line.replace('\\', '')
        if line.startswith('# '):
            line = f'[bold color(231)]{line.replace("# ", "")}[/bold color(231)]'
        code_number = line.count('`')
        state_even = False
        for _ in range(code_number):
            if not state_even:
                line = line.replace('`', '[bold yellow]', 1)
                state_even = True
            else:
                line = line.replace('`', '[/bold yellow]', 1)
                state_even = False
        print(f'[not bold]{line}[/not bold]')
    sys.exit(0)


def print_logos() -> None:
    for i, logo in enumerate(logos):
        print(f'logo {i + 1}:\n{logo}')
    sys.exit(0)


def clamp(inp: int, low: int, high: int) -> int:
    return min(max(inp, low), high)


def trim_names(fl: str, compensate: int) -> str:
    if len(fl) > 40 - compensate:
        fl = f'{fl[0:37 - compensate]}...'
    return fl.ljust(40 - compensate, ' ')


def stamp_to_sec(stamp):
    l = stamp.split(':')
    return int(l[0])*3600 + int(l[1])*60 + float(l[2])


def convert_delay_to_ms(inp, compensate):
    if not inp.startswith(('-', '+', 'm', 'p')): print_exit('delay')
    inp = inp.replace(',', '.')

    negative = False
    if inp.startswith(('-', 'm')): negative = True

    if '@' in inp:
        frame = round(float(re.sub('[^0-9\.]', '', inp.split('@')[0])))
        if not frame: print_exit('delay')

        fps = str(inp.split('@')[1])
        if fps == 'ntsc': fps = str(24000 / 1001)
        if fps == 'pal': fps = str(25)
        if '/' in fps:
            divident = re.sub('[^0-9\.]', '', fps.split('/')[0])
            divisor = re.sub('[^0-9\.]', '', fps.split('/')[1])
            if not divident or not divisor: print_exit('delay')
            fps = float(divident) / float(divisor)
        delay = frame / float(fps) * 1000
    else:
        if not inp.endswith(('s', 'ms')): print_exit('delay')
        if not inp.count('.') < 2: print_exit('delay')

        if inp.endswith('ms'):
            delay = float(re.sub('[^0-9\.]', '', inp))
        else:
            delay = float(re.sub('[^0-9\.]', '', inp)) * 1000

    delay = float(f'-{delay}' if negative else f'+{delay}')
    delay_print = f'{format(delay, ".3f")} ms'

    if compensate: delay -= 16 / 3 # 256 / 48000 * 1000

    delay_mode = 'prepend_silence_duration'
    if delay < 0:
        delay_mode = 'start'
        delay_xml = str(timedelta(seconds = (abs(delay) / 1000)))
        if '.' not in delay_xml:
            delay_xml = f'{delay_xml}.0'
    else:
        delay_xml = format(delay / 1000, ".6f")

    return delay_print, delay_xml, delay_mode


def channel_number_to_name(inp: int):
    channel_names = {
        1: 'mono',
        2: 'stereo',
        6: '5.1',
        8: '7.1'
    }
    return channel_names[inp]


def find_closest_allowed(value: int, allowed_values: list[int]) -> int:
    return min(allowed_values, key=lambda list_value: abs(list_value - value))


def wpc(p: str) -> str:
    if config['wsl']:
        if not p.startswith('/mnt/'): print_exit('wsl_path', p)
        parts = p.split('/')[2:]
        parts[0] = parts[0].upper() + ':'
        p = '\\'.join(parts)
    return p


def save_xml(f: str, xml: dict[str, Any]) -> None:
    with open(f, 'w') as fd:
        fd.write(xmltodict.unparse(xml, pretty=True, indent='  '))


def basename(fl: str, format_: str) -> str:
    return os.path.basename(os.path.splitext(fl)[0]) + f'.{format_}'


def print_exit(message: str, insert: Any = None) -> NoReturn:
    if insert and '🤠' in error_messages[message]:
        message_split = error_messages[message].split('🤠')
        before, after = message_split[0], message_split[1]
        exit_message = f'[red]ERROR:[/red] {before}{insert}{after}'
    else:
        exit_message = f'[red]ERROR:[/red] {error_messages[message]}'
    print(exit_message)
    sys.exit(1)


def createdir(out: str) -> None:
    try:
        os.makedirs(out, exist_ok=True)
    except OSError:
        print_exit('create_dir', out)


def encode(task_id: TaskID, settings: list) -> None:
    fl, output, length, ffmpeg_args, dee_args, intermediate_exists, aformat = settings
    fl_b = os.path.basename(fl)
    pb.update(description=f'[bold][cyan]starting[/cyan][/bold]...{" " * 24}', task_id=task_id, visible=True)

    if not intermediate_exists:
        if length == -1:
            pb.update(description=f'[bold][cyan]ffmpeg[/cyan][/bold] | {trim_names(fl_b, 6)}', task_id=task_id, total=None)
            ffmpeg = subprocess.run(ffmpeg_args, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, universal_newlines=True, encoding='utf-8', errors='ignore')
        else:
            pb.update(description=f'[bold][cyan]ffmpeg[/cyan][/bold] | {trim_names(fl_b, 6)}', task_id=task_id, total=100)
            ffmpeg = subprocess.Popen(ffmpeg_args, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, universal_newlines=True, encoding='utf-8', errors='ignore')
            percentage_length = length / 100
            with ffmpeg.stdout:
                for line in iter(ffmpeg.stdout.readline, ''):
                    if '=' not in line: continue
                    progress = re.search(r'time=([^\s]+)', line)
                    if progress:
                        timecode = stamp_to_sec(progress[1]) / percentage_length
                        pb.update(task_id=task_id, completed=timecode)
        pb.update(task_id=task_id, completed=100)
        time.sleep(0.5)

    if args.dialnorm != 0 and aformat == 'thd':
        pb.update(description=f'[bold][cyan]dee[/cyan][/bold]: encode | {trim_names(fl_b, 11)}', task_id=task_id, completed=0, total=100)
    else:
        pb.update(description=f'[bold][cyan]dee[/cyan][/bold]: measure | {trim_names(fl_b, 12)}', task_id=task_id, completed=0, total=100)
    dee = subprocess.Popen(dee_args, stdout=subprocess.PIPE, stderr=subprocess.PIPE, universal_newlines=True, encoding='utf-8', errors='ignore')
    with dee.stdout:
        encoding_step = False
        for line in iter(dee.stdout.readline, ''):
            if not encoding_step and re.search(r'(measured_loudness|speech gated loudness)', line):
                encoding_step = True
                measured_dn = re.search(r'(measured_loudness|speech\ gated\ loudness)(\=|\:\ )(-?.+)', line)
                measured_dn = round(float(measured_dn[3].strip('.')))
                measured_dn = str(clamp(measured_dn, -31, 0))
                if args.measure_only:
                    pb.update(description=f'[bold][cyan]dee[/cyan][/bold]: measure | {trim_names(fl_b, 18 + len(measured_dn))} ({measured_dn} dB)', task_id=task_id, completed=100)
                    dee.kill()
                else:
                    pb.update(description=f'[bold][cyan]dee[/cyan][/bold]: encode | {trim_names(fl_b, 17 + len(measured_dn))} ({measured_dn} dB)', task_id=task_id)

            progress = re.search(r'Overall progress: ([0-9]+\.[0-9])', line)
            if progress: pb.update(task_id=task_id, completed=float(progress[1]))

            if 'error' in line.lower():
                print(line.rstrip().split(': ', 1)[1])
    pb.update(task_id=task_id, completed=100)

    if not args.keeptemp:
        os.remove(os.path.join(config['temp_path'], basename(fl, 'wav')))
        os.remove(os.path.join(config['temp_path'], basename(fl, 'xml')))

    if args.format.lower() == 'thd':
        os.remove(os.path.join(output, basename(fl, 'thd.log')))
        os.remove(os.path.join(output, basename(fl, 'thd.mll')))


def main() -> None:
    if len(sys.argv) == 1:
        parser.print_help(sys.stderr)
        sys.exit(1)

    if args.changelog: print_changelog()

    if args.printlogos: print_logos()
    if 0 < config['logo'] < len(logos) + 1: print(logos[config['logo'] - 1])

    if args.threads:
        threads = clamp(args.threads, 1, cpu_count() - 2)
    else:
        threads = clamp(config['threads'], 1, cpu_count() - 2)

    aformat = args.format.lower()
    bitrate = args.bitrate
    downmix = args.downmix
    args.dialnorm = clamp(args.dialnorm, -31, 0)

    if aformat not in ['dd', 'ddp', 'thd']: print_exit('format')
    if downmix and downmix not in [1, 2, 6]: print_exit('downmix')
    if downmix and aformat == 'thd': print_exit('thd_downmix')
    if args.drc not in ['film_light', 'film_standard', 'music_light', 'music_standard', 'speech']: print_exit('drc')
    if platform.system() == 'Linux' and not config['wsl'] and aformat == 'thd': print_exit('linux_thd')
    if args.measure_only: aformat = 'ddp'

    filelist = []
    for f in args.input:
        if not os.path.exists(f): print_exit('path', f)
        if os.path.isdir(f):
            filelist.extend(glob(f + os.path.sep + '*'))
        else:
            filelist.append(f)

    samplerate_list = []
    channels_list = []
    bit_depth_list = []
    length_list = []

    for f in filelist:
        probe_args = [config["ffprobe_path"], '-v', 'quiet', '-select_streams', 'a:0', '-print_format', 'json', '-show_format', '-show_streams', f]
        try:
            output = subprocess.check_output(probe_args, encoding='utf-8')
        except subprocess.CalledProcessError:
            print_exit('ffprobe')
        audio = json.loads(output)['streams'][0]
        samplerate_list.append(int(audio['sample_rate']))
        channels_list.append(audio['channels'])
        length_list.append(float(audio.get('duration', -1)))
        depth = int(audio.get('bits_per_sample', 0))
        if depth == 0: depth = int(audio.get('bits_per_raw_sample', 32))
        bit_depth_list.append(depth)

    if not samplerate_list.count(samplerate_list[0]) == len(samplerate_list): print_exit('sample_mismatch', samplerate_list)
    if not channels_list.count(channels_list[0]) == len(channels_list): print_exit('channel_mismatch', channels_list)
    if not bit_depth_list.count(bit_depth_list[0]) == len(bit_depth_list): print_exit('bitdepth_mismatch', bit_depth_list)

    channels = channels_list[0]
    samplerate = samplerate_list[0]
    bit_depth = bit_depth_list[0]
    if bit_depth not in [16, 24, 32]:
        if bit_depth < 16:
            bit_depth = 16
        elif 16 < bit_depth < 24:
            bit_depth = 24
        else:
            bit_depth = 32

    if channels not in [1, 2, 6, 8]: print_exit('channels')
    if downmix and downmix >= channels: print_exit('downmix_mismatch')
    if not downmix and aformat == 'dd' and channels == 8: downmix = 6

    downmix_config = 'off'
    if downmix:
        outchannels = downmix
        downmix_config = channel_number_to_name(outchannels)
    else:
        outchannels = channels

    if outchannels in [1, 2]:
        if args.no_prompt:
            print('Consider using [bold cyan]qaac[/bold cyan] or [bold cyan]opus[/bold cyan] for [bold yellow]mono[/bold yellow] and [bold yellow]stereo[/bold yellow] encoding.')
        else:
            continue_enc = Confirm.ask('Consider using [bold cyan]qaac[/bold cyan] or [bold cyan]opus[/bold cyan] for [bold yellow]mono[/bold yellow] and [bold yellow]stereo[/bold yellow] encoding, are you sure you want to use [bold cyan]dee[/bold cyan]?')
            if not continue_enc: sys.exit(1)

    if args.dialnorm != 0:
        if args.no_prompt:
            print('Consider leaving the dialnorm value at 0 (auto), setting it manually can be dangerous.')
        else:
            continue_enc = Confirm.ask('Consider leaving the dialnorm value at 0 (auto), setting it manually can be dangerous, are you sure you want to do it?')
            if not continue_enc: sys.exit(1)

    if aformat == 'dd':
        if outchannels == 1:
            if not bitrate: bitrate = 128
            bitrate = find_closest_allowed(bitrate, allowed_bitrates['dd_10'])
        elif outchannels == 2:
            if not bitrate: bitrate = 256
            bitrate = find_closest_allowed(bitrate, allowed_bitrates['dd_20'])
        elif outchannels == 6:
            if not bitrate: bitrate = 640
            bitrate = find_closest_allowed(bitrate, allowed_bitrates['dd_51'])
    elif aformat == 'ddp':
        if outchannels == 1:
            if not bitrate: bitrate = 128
            bitrate = find_closest_allowed(bitrate, allowed_bitrates['ddp_10'])
        elif outchannels == 2:
            if not bitrate: bitrate = 256
            bitrate = find_closest_allowed(bitrate, allowed_bitrates['ddp_20'])
        elif outchannels == 6:
            if not bitrate: bitrate = 1024
            bitrate = find_closest_allowed(bitrate, allowed_bitrates['ddp_51'])
        elif outchannels == 8:
            if not bitrate: bitrate = 1536
            bitrate = find_closest_allowed(bitrate, allowed_bitrates['ddp_71'])

    if args.output:
        createdir(os.path.abspath(args.output))
        output = os.path.abspath(args.output)
    else:
        output = os.getcwd()

    if aformat in ['dd', 'ddp']:
        xml_base = xmltodict.parse(xml_dd_ddp_base)
        xml_base['job_config']['output']['ec3']['storage']['local']['path'] = f'\"{wpc(output)}\"'
        if aformat == 'ddp':
            xml_base['job_config']['filter']['audio']['pcm_to_ddp']['encoder_mode'] = 'ddp'
            if outchannels == 8:
                xml_base['job_config']['filter']['audio']['pcm_to_ddp']['encoder_mode'] = 'ddp71'
                if bitrate > 1024:
                    xml_base['job_config']['filter']['audio']['pcm_to_ddp']['encoder_mode'] = 'bluray'
        if aformat == 'dd':
            xml_base['job_config']['filter']['audio']['pcm_to_ddp']['encoder_mode'] = 'dd'
        xml_base['job_config']['filter']['audio']['pcm_to_ddp']['downmix_config'] = downmix_config
        xml_base['job_config']['filter']['audio']['pcm_to_ddp']['data_rate'] = bitrate
        xml_base['job_config']['filter']['audio']['pcm_to_ddp']['drc']['line_mode_drc_profile'] = args.drc
        xml_base['job_config']['filter']['audio']['pcm_to_ddp']['drc']['rf_mode_drc_profile'] = args.drc
        xml_base['job_config']['filter']['audio']['pcm_to_ddp']['custom_dialnorm'] = args.dialnorm
        delay_print, delay_xml, delay_mode = convert_delay_to_ms(args.delay, compensate=True)
        xml_base['job_config']['filter']['audio']['pcm_to_ddp'][delay_mode] = delay_xml
    elif aformat == 'thd':
        xml_base = xmltodict.parse(xml_thd_base)
        xml_base['job_config']['output']['mlp']['storage']['local']['path'] = f'\"{wpc(output)}\"'
        xml_base['job_config']['filter']['audio']['encode_to_dthd']['atmos_presentation']['drc_profile'] = args.drc
        xml_base['job_config']['filter']['audio']['encode_to_dthd']['presentation_8ch']['drc_profile'] = args.drc
        xml_base['job_config']['filter']['audio']['encode_to_dthd']['presentation_6ch']['drc_profile'] = args.drc
        xml_base['job_config']['filter']['audio']['encode_to_dthd']['custom_dialnorm'] = args.dialnorm
        delay_print, delay_xml, delay_mode = convert_delay_to_ms(args.delay, compensate=False)
        xml_base['job_config']['filter']['audio']['encode_to_dthd'][delay_mode] = delay_xml

    xml_base['job_config']['input']['audio']['wav']['storage']['local']['path'] = f'\"{wpc(config["temp_path"])}\"'
    xml_base['job_config']['misc']['temp_dir']['path'] = f'\"{wpc(config["temp_path"])}\"'

    if config['show_summary']:
        try:
            latest_version = requests.get('https://raw.githubusercontent.com/pcroland/deew/main/version.py').text.split("'")[1]
            if version.parse(prog_version) < version.parse(latest_version):
                latest_version = f'[bold green]{latest_version}[/bold green] !!!'
        except Exception:
            latest_version = "[red]couldn't fetch"
        summary = Table(title='[not italic bold magenta]Encoding summary')
        summary.add_column('[bold cyan]Version\nLatest', style='green')
        summary.add_column(f'[bold color(231)]{prog_version}\n{latest_version}', style='color(231)')

        summary.add_row('[bold yellow]Output')
        summary.add_row('Format', 'TrueHD' if aformat == 'thd' else aformat.upper())
        summary.add_row('Channels', channel_number_to_name(outchannels))
        summary.add_row('Bitrate', 'N/A' if aformat == 'thd' else f'{str(bitrate)} kbps')
        summary.add_row('Dialnorm', 'auto (0)' if args.dialnorm == 0 else f'{str(args.dialnorm)} dB', end_section=True)

        summary.add_row('[bold yellow]Input')
        summary.add_row('Channels', channel_number_to_name(channels))
        summary.add_row('Bit depth', str(bit_depth), end_section=True)
        summary.add_row('[bold yellow]Other')
        summary.add_row('Files', str(len(filelist)))
        summary.add_row('Max threads', str(threads))
        summary.add_row('Delay', delay_print)
        print(summary)
        print()

    resample_value = ''
    if aformat in ['dd', 'ddp'] and samplerate != 48000:
        bit_depth = 32
        resample_value = '48000'
    elif aformat == 'thd' and samplerate not in [48000, 96000]:
        bit_depth = 32
        if samplerate < 72000:
            resample_value = '48000'
        else:
            resample_value = '96000'
    if resample_value:
        if channels == 8:
            channel_swap = 'pan=7.1|c0=c0|c1=c1|c2=c2|c3=c3|c4=c6|c5=c7|c6=c4|c7=c5,'
        else:
            channel_swap = ''
        resample_args = ['-filter_complex', f'{channel_swap}aresample=resampler=soxr', '-ar', resample_value, '-precision', '28', '-cutoff', '1', '-dither_scale', '0']
        resample_args_print = f'-filter_complex [bold color(231)]{channel_swap}aresample=resampler=soxr[/bold color(231)] -ar [bold color(231)]{resample_value}[/bold color(231)] -precision [bold color(231)]28[/bold color(231)] -cutoff [bold color(231)]1[/bold color(231)] -dither_scale [bold color(231)]0[/bold color(231)] '
    else:
        resample_args = []
        resample_args_print = ''

    if channels == 8 and not resample_args:
        channel_swap_args = ['-filter_complex', 'pan=7.1|c0=c0|c1=c1|c2=c2|c3=c3|c4=c6|c5=c7|c6=c4|c7=c5']
        channel_swap_args_print = '-filter_complex [bold color(231)]pan=7.1|c0=c0|c1=c1|c2=c2|c3=c3|c4=c6|c5=c7|c6=c4|c7=c5[/bold color(231)] '
    else:
        channel_swap_args = []
        channel_swap_args_print = ''

    if config['wsl'] or platform.system() == 'Windows':
        dee_xml_input_base = f'{wpc(config["temp_path"])}\\'
    else:
        dee_xml_input_base = f'{config["temp_path"]}/'

    xml_validation = []
    xml_validation_print = ''
    if aformat != 'thd' and bitrate > 1024 and platform.system() != 'Windows' and not config['wsl']:
        xml_validation = ['--disable-xml-validation']
        xml_validation_print = ' --disable-xml-validation'

    settings = []
    ffmpeg_print_list = []
    dee_print_list = []
    intermediate_exists_list = []
    for i in range(len(filelist)):
        dee_xml_input = f'{dee_xml_input_base}{basename(filelist[i], "xml")}'

        ffmpeg_args = [config['ffmpeg_path'], '-y', '-drc_scale', '0', '-i', filelist[i], '-c:a:0', f'pcm_s{bit_depth}le', *(channel_swap_args), *(resample_args), '-rf64', 'always', os.path.join(config['temp_path'], basename(filelist[i], 'wav'))]
        dee_args = [config['dee_path'], '--progress-interval', '500', '--diagnostics-interval', '90000', '-x', dee_xml_input, *(xml_validation)]

        ffmpeg_args_print = f'[bold cyan]ffmpeg[/bold cyan] -y -drc_scale [bold color(231)]0[/bold color(231)] -i [bold green]{filelist[i]}[/bold green] [not bold white]-c:a[/not bold white]' + f'[not bold white]:0[/not bold white] [bold color(231)]pcm_s{bit_depth}le[/bold color(231)] {channel_swap_args_print}{resample_args_print}-rf64 [bold color(231)]always[/bold color(231)] [bold magenta]{os.path.join(config["temp_path"], basename(filelist[i], "wav"))}[/bold magenta]'
        dee_args_print = f'[bold cyan]dee[/bold cyan] -x [bold magenta]{dee_xml_input}[/bold magenta]{xml_validation_print}'
        ffmpeg_args_print_short = f'[bold cyan]ffmpeg[/bold cyan] -y -drc_scale [bold color(231)]0[/bold color(231)] -i [bold green]\[input][/bold green] [not bold white]-c:a[/not bold white]' + f'[not bold white]:0[/not bold white] [bold color(231)]pcm_s{bit_depth}le[/bold color(231)] {channel_swap_args_print}{resample_args_print}-rf64 [bold color(231)]always[/bold color(231)] [bold magenta]\[output][/bold magenta]'
        dee_args_print_short = f'[bold cyan]dee[/bold cyan] -x [bold magenta]\[input][/bold magenta]{xml_validation_print}'

        intermediate_exists = False
        if os.path.exists(os.path.join(config['temp_path'], basename(filelist[i], 'wav'))):
            intermediate_exists = True
            ffmpeg_args_print = '[green]Intermediate already exists[/green]'

        ffmpeg_print_list.append(ffmpeg_args_print)
        dee_print_list.append(dee_args_print)
        if intermediate_exists: intermediate_exists_list.append(filelist[i])

        xml = deepcopy(xml_base)
        xml['job_config']['input']['audio']['wav']['file_name'] = f'\"{basename(filelist[i], "wav")}\"'
        if aformat == 'ddp':
            if bitrate > 1024:
                xml['job_config']['output']['ec3']['file_name'] = f'\"{basename(filelist[i], "eb3")}\"'
            else:
                xml['job_config']['output']['ec3']['file_name'] = f'\"{basename(filelist[i], "ec3")}\"'
        elif aformat == 'dd':
            xml['job_config']['output']['ec3']['file_name'] = f'\"{basename(filelist[i], "ac3")}\"'
            xml['job_config']['output']['ac3'] = xml['job_config']['output']['ec3']
            del xml['job_config']['output']['ec3']
        else:
            xml['job_config']['output']['mlp']['file_name'] = f'\"{basename(filelist[i], "thd")}\"'
        save_xml(os.path.join(config['temp_path'], basename(filelist[i], 'xml')), xml)

        settings.append([filelist[i], output, length_list[i], ffmpeg_args, dee_args, intermediate_exists, aformat])

    if args.long_argument:
        print('[bold color(231)]Running the following commands:[/bold color(231)]')
        for ff, d in zip(ffmpeg_print_list, dee_print_list):
            print(f'{ff} && {d}')
    else:
        if intermediate_exists_list:
            print(f'[bold color(231)]Intermediate already exists for the following file(s):[/bold color(231)] [bold magenta]{"[not bold white],[/not bold white] ".join(intermediate_exists_list)}[bold magenta]')
        print(f'[bold color(231)]Running the following commands:[/bold color(231)]\n{ffmpeg_args_print_short} && {dee_args_print_short}')

    print()

    with pb:
        with ThreadPoolExecutor(max_workers=threads) as pool:
            for setting in settings:
                task_id = pb.add_task('', visible=False, total=None)
                pool.submit(encode, task_id, setting)


if __name__ == '__main__':
    if getattr(sys, 'frozen', False):
        script_path = os.path.dirname(sys.executable)
    else:
        script_path = os.path.dirname(__file__)

    if os.path.exists(os.path.join(script_path, 'config.toml')):
        config = toml.load(os.path.join(script_path, 'config.toml'))
    elif platform.system() == 'Linux' and os.path.exists(
            os.path.join(os.path.expanduser('~'), '.config', 'deew', 'config.toml')):
        config = toml.load(os.path.join(os.path.expanduser('~'), '.config', 'deew', 'config.toml'))
    else:
        print_exit('config')

    if not config['temp_path']:
        config['temp_path'] = os.path.join(script_path, 'temp')
    config['temp_path'] = os.path.abspath(config['temp_path'])
    createdir(config['temp_path'])

    if not shutil.which(config['dee_path']): print_exit('binary_exist', config['dee_path'])
    if not shutil.which(config['ffmpeg_path']): print_exit('binary_exist', config['ffmpeg_path'])
    if not shutil.which(config['ffprobe_path']): print_exit('binary_exist', config['ffprobe_path'])

    pb = Progress('[', '{task.description}', ']', BarColumn(), '[magenta]{task.percentage:>3.2f}%', refresh_per_second=8)

    signal.signal(signal.SIGINT, signal.SIG_DFL)
    main()
