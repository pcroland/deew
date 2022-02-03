#!/usr/bin/env python3
import argparse
import os
import platform
import signal
import subprocess
import sys
from glob import glob
from multiprocessing import Pool, cpu_count

import toml
import xmltodict
from tqdm import tqdm

from logos import logos

signal.signal(signal.SIGINT, signal.SIG_DFL)

parser = argparse.ArgumentParser(add_help=False, formatter_class=argparse.RawTextHelpFormatter)
parser.add_argument('-h', '--help',
                    action='help',
                    default=argparse.SUPPRESS,
                    help='shows this help message.')
parser.add_argument('-v', '--version',
                    action='version',
                    version='deew 1.0',
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
parser.add_argument('-c', '--channels',
                    default='6',
                    help='default: 6')
parser.add_argument('-d', '--dialnorm',
                    type=int,
                    default=-31,
                    help='default: -31')
parser.add_argument('-t', '--threads',
                    type=int,
                    default=cpu_count() - 1,
                    help='number of threads to use.\ndefault: all threads-1')
parser.add_argument('-k', '--keeptemp',
                    action='store_true',
                    help='keep temp files')
parser.add_argument('-p', '--progress',
                    action='store_true',
                    help='use progress bar instead of command printing')
parser.add_argument('--printlogos',
                    action='store_true',
                    help='show all logo variants you can set in the config')
args = parser.parse_args()

def printlogos():
    for i in range(len(logos)):
        print(f'logo {i + 1}:\n{logos[i]}')
    sys.exit(1)

def clamp(inp, low, high):
    return min(max(inp, low), high)

def wpc(p):
    p = p.split('/')[2:]
    p[0] = p[0].upper() + ':'
    return '\\'.join(p)

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

    xml['job_config']['input']['audio']['wav']['file_name'] = f'\"{basename(fl, "wav")}\"'

    if args.format.lower() == 'ddp':
        if args.channels == '8':
            xml['job_config']['filter']['audio']['pcm_to_ddp']['encoder_mode'] = "bluray"
            xml['job_config']['filter']['audio']['pcm_to_ddp']['data_rate'] = '1536' if args.bitrate == 0 else args.bitrate
            xml['job_config']['output']['ec3']['file_name'] = f'\"{basename(fl, "eb3")}\"'
        else:
            xml['job_config']['filter']['audio']['pcm_to_ddp']['encoder_mode'] = "ddp"
            xml['job_config']['filter']['audio']['pcm_to_ddp']['data_rate'] = '1024' if args.bitrate == 0 else args.bitrate
            xml['job_config']['output']['ec3']['file_name'] = f'\"{basename(fl, "ec3")}\"'
    elif args.format.lower() == 'dd':
        xml['job_config']['filter']['audio']['pcm_to_ddp']['encoder_mode'] = 'dd'
        xml['job_config']['filter']['audio']['pcm_to_ddp']['data_rate'] = '640' if args.bitrate == 0 else args.bitrate
        xml['job_config']['output']['ec3']['file_name'] = f'\"{basename(fl, "ac3")}\"'
        xml['job_config']['output']['ac3'] = xml['job_config']['output']['ec3']
        del xml['job_config']['output']['ec3']
    else:
        xml['job_config']['output']['mlp']['file_name'] = f'\"{basename(fl, "thd")}\"'
    savexml(os.path.join(config['temp_path'], basename(fl, 'xml')), xml)

    ffmpeg_args = [config['ffmpeg_path'], '-y', '-i', fl, '-ac', args.channels, '-c:a', 'pcm_s24le', '-rf64', 'always', os.path.join(config['temp_path'], basename(fl, 'wav'))]
    dee_args = [config['dee_path'], '-x', os.path.join(wpc(config['temp_path']) if config['wsl'] == 1 else config['temp_path'], basename(fl, 'xml'))]
    if not args.progress: print(' '.join(ffmpeg_args) + ' && ' +' '.join(dee_args))
    subprocess.run(ffmpeg_args, stderr=subprocess.DEVNULL, stdout=subprocess.DEVNULL)
    subprocess.run(dee_args, stderr=subprocess.DEVNULL, stdout=subprocess.DEVNULL)

    if not args.keeptemp:
        os.remove(os.path.join(config['temp_path'], basename(fl, 'wav')))
        os.remove(os.path.join(config['temp_path'], basename(fl, 'xml')))

    if args.format.lower() == 'thd':
        os.remove(os.path.join(os.getcwd(), basename(fl, 'thd.log')))
        os.remove(os.path.join(os.getcwd(), basename(fl, 'thd.mll')))

script_path = os.path.dirname(__file__)
config = opentoml(os.path.join(script_path, 'config.toml'))
if platform.system() == 'Windows': config['wsl'] = 0

def main():
    if len(sys.argv) == 1:
        parser.print_help(sys.stderr)
        sys.exit(1)

    if args.printlogos: printlogos()
    if 0 < config['logo'] < len(logos) + 1: print(logos[config['logo'] - 1])

    if args.format.lower() not in ['dd', 'ddp', 'thd']: printexit('ERROR: format has to be dd, ddp or thd.')
    if args.channels not in ['6', '8']: printexit('ERROR: channels has to be 6 or 8.')
    if args.format.lower() == 'ddp' and args.channels == '8' and args.bitrate > 1664: printexit('ERROR: bitrate for 7.1 ddp format has to be 1664 or lower.')
    if args.format.lower() == 'ddp' and args.channels == '6' and args.bitrate > 1024: printexit('ERROR: bitrate for 5.1 ddp format has to be 1024 or lower.')
    if args.format.lower() == 'dd' and (args.channels == '8' or args.bitrate > 640): printexit('ERROR: channels for dd format has to be 6 and bitrate has to be 640 or lower.')

    if args.format.lower() in ['dd', 'ddp']:
        xmlbase = openxml(os.path.join(script_path, 'xml', 'ddp.xml'))
        xmlbase['job_config']['output']['ec3']['storage']['local']['path'] = f'\"{wpc(os.getcwd()) if config["wsl"] == 1 else os.getcwd()}\"'
        xmlbase['job_config']['filter']['audio']['pcm_to_ddp']['custom_dialnorm'] = clamp(args.dialnorm, -31, 0)
    else:
        xmlbase = openxml(os.path.join(script_path, 'xml', 'thd.xml'))
        xmlbase['job_config']['output']['mlp']['storage']['local']['path'] = f'\"{wpc(os.getcwd()) if config["wsl"] == 1 else os.getcwd()}\"'

    xmlbase['job_config']['input']['audio']['wav']['storage']['local']['path'] = f'\"{wpc(config["temp_path"]) if config["wsl"] == 1 else config["temp_path"]}\"'
    xmlbase['job_config']['misc']['temp_dir']['path'] = f'\"{wpc(config["temp_path"]) if config["wsl"] == 1 else config["temp_path"]}\"'

    threads = clamp(args.threads, 1, cpu_count() - 1)
    pool = Pool(threads)

    filelist = []
    for f in args.input:
        if os.path.isdir(f):
            filelist.extend(glob(f + os.path.sep + '*'))
        else:
            filelist.append(f)

    settings = []
    for i in range(len(filelist)):
        settings.append([filelist[i], xmlbase])

    if args.progress:
        for audio in tqdm(pool.imap_unordered(encode, settings), total=len(args.input), desc='encode', bar_format='{l_bar}{bar}| {n_fmt}/{total_fmt} audio done. '):
            pass
    else:
        for audio in pool.imap_unordered(encode, settings):
            pass

if __name__ == "__main__":
    main()
# compensate for encoding offset
