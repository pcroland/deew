#!/usr/bin/env python3

from __future__ import annotations

import json
import ntpath
import os
import platform
import re
import shutil
import signal
import subprocess
import sys
import tempfile
from concurrent.futures import ThreadPoolExecutor
from copy import deepcopy
from glob import glob
from multiprocessing import cpu_count

import requests
import toml
import xmltodict
from packaging import version
from platformdirs import PlatformDirs
from rich import print
from rich.console import Console
from rich.progress import BarColumn, Progress
from rich.prompt import Confirm
from rich.syntax import Syntax
from rich.table import Table

# TODO this is generally frowned upon in Python, although
# in rare circumstances okay for us to use.
# We can likely achieve what you was wanting ultimately
# by using the function _get_working_dir() inside
# deew > deew2 > utils > utils.py
sys.path.append(".")

from deew.bitrates import allowed_bitrates
from deew.logos import logos
from deew.xml import xml_dd_ddp_base, xml_thd_base, xml_ac4_base
from cli.utils import (
    print_changelog,
    list_bitrates,
    print_logos,
    generate_config,
    createdir,
    parse_version_string,
)
from deew._version import ProgramInfo
from deew.payloads import DeePayload

from deew.utils import (
    basename,
    channel_number_to_name,
    clamp,
    convert_delay_to_ms,
    find_closest_allowed,
    rwpc,
    wpc,
)

from deew.xml.utils import save_xml

from deew.encode import encode

from deew.simplens import simplens

# temp
from cli.utils import print_exit


def main(payload: DeePayload) -> None:
    if payload.changelog:
        print_changelog()
    if payload.list_bitrates:
        list_bitrates()
    if payload.print_logos:
        print_logos()

    if getattr(sys, "frozen", False):
        script_path = os.path.dirname(sys.executable)
        standalone = 1
    else:
        script_path = os.path.dirname(__file__)
        standalone = 0

    dirs = PlatformDirs("deew", False)
    config_dir_path = dirs.user_config_dir
    config_path1 = os.path.join(config_dir_path, "config.toml")
    config_path2 = os.path.join(script_path, "config.toml")

    if payload.config:
        if standalone:
            print(
                f"[bold cyan]Your config locations:[/bold cyan]\n{config_path1}\n{config_path2}\n\n[bold cyan]Your current config:[/bold cyan]"
            )
            if os.path.exists(config_path1):
                current_conf = config_path1
            elif os.path.exists(config_path2):
                current_conf = config_path2
            else:
                print("You don't have a config currently.")
                sys.exit(0)
        else:
            print(
                f"[bold cyan]Your config location:[/bold cyan]\n{config_path1}\n\n[bold cyan]Your current config:[/bold cyan]"
            )
            if os.path.exists(config_path1):
                current_conf = config_path1
            else:
                print("You don't have a config currently.")
                sys.exit(0)
        with open(current_conf, "r") as conf:
            Console().print(Syntax(conf.read(), "toml"))
        sys.exit(0)

    if payload.generate_config:
        generate_config(standalone, config_path1, config_path2, config_dir_path)
        sys.exit(0)

    if not os.path.exists(config_path1) and not os.path.exists(config_path2):
        print(
            f"[bold yellow]config.toml[/bold yellow] [not bold white]is missing, creating one...[/not bold white]"
        )
        generate_config(standalone, config_path1, config_path2, config_dir_path)

    try:
        config = toml.load(config_path1)
    except Exception:
        config = toml.load(config_path2)
    simplens.config = config

    if 0 < config["logo"] < len(logos) + 1:
        print(logos[config["logo"] - 1])

    config_keys = [
        "ffmpeg_path",
        "ffprobe_path",
        "dee_path",
        "temp_path",
        "logo",
        "max_instances",
        "default_bitrates",
        "summary_sections",
    ]
    c_key_missing = []
    for c_key in config_keys:
        if c_key not in config:
            c_key_missing.append(c_key)
    if len(c_key_missing) > 0:
        print_exit(
            "config_key",
            f'[bold yellow]{"[not bold white], [/not bold white]".join(c_key_missing)}[/bold yellow]',
        )

    for i in config["dee_path"], config["ffmpeg_path"], config["ffprobe_path"]:
        if not shutil.which(i):
            print_exit("binary_exist", i)

    with open(shutil.which(config["dee_path"]), "rb") as fd:
        simplens.dee_is_exe = fd.read(2) == b"\x4d\x5a"
    simplens.is_nonnative_exe = simplens.dee_is_exe and platform.system() != "Windows"

    if not config["temp_path"]:
        if simplens.is_nonnative_exe:
            config["temp_path"] = rwpc(
                ntpath.join(
                    subprocess.run(
                        ["powershell.exe", "(gi $env:TEMP).fullname"],
                        capture_output=True,
                        encoding="utf-8",
                    ).stdout.strip(),
                    "deew",
                )
            )
        else:
            config["temp_path"] = (
                os.path.join(script_path, "temp")
                if standalone
                else tempfile.gettempdir()
            )
            if config["temp_path"] == "/tmp":
                config["temp_path"] = "/var/tmp/deew"
    config["temp_path"] = os.path.abspath(config["temp_path"])
    createdir(config["temp_path"])

    cpu__count = cpu_count()
    if payload.instances:
        instances = payload.instances
    else:
        instances = config["max_instances"]
    if isinstance(instances, str) and instances.endswith("%"):
        instances = cpu__count * (int(instances.replace("%", "")) / 100)
    else:
        instances = int(instances)
    if simplens.dee_is_exe:
        instances = clamp(instances, 1, cpu__count - 2)
        instances = clamp(instances, 1, 6)
    else:
        instances = clamp(instances, 1, cpu__count)
    if instances == 0:
        instances = 1

    aformat = payload.encoder_format.lower()
    bitrate = payload.bitrate
    downmix = payload.downmix
    payload.dialnorm = clamp(payload.dialnorm, -31, 0)
    trackindex = max(0, payload.track_index)

    if aformat not in ["dd", "ddp", "thd", "ac4"]:
        print_exit("format")
    if downmix and downmix not in [1, 2, 6]:
        print_exit("downmix")
    if downmix and aformat == "thd":
        print_exit("thd_downmix")
    if payload.drc not in [
        "film_light",
        "film_standard",
        "music_light",
        "music_standard",
        "speech",
        "none",
    ]:
        print_exit("drc")
    if not simplens.dee_is_exe and platform.system() == "Linux" and aformat == "thd":
        print_exit("linux_thd")
    if payload.measure_only:
        aformat = "ddp"

    filelist = []
    for f in payload.file_input:
        if not os.path.exists(f):
            print_exit("path", f)
        if os.path.isdir(f):
            filelist.extend(glob(f + os.path.sep + "*"))
        else:
            filelist.append(f)

    samplerate_list = []
    channels_list = []
    bit_depth_list = []
    length_list = []

    for f in filelist:
        probe_args = [
            config["ffprobe_path"],
            "-v",
            "quiet",
            "-select_streams",
            f"a:{trackindex}",
            "-print_format",
            "json",
            "-show_format",
            "-show_streams",
            f,
        ]
        try:
            output = subprocess.check_output(probe_args, encoding="utf-8")
        except subprocess.CalledProcessError:
            print_exit("ffprobe")
        audio = json.loads(output)["streams"][0]
        samplerate_list.append(int(audio["sample_rate"]))
        channels_list.append(audio["channels"])
        length_list.append(float(audio.get("duration", -1)))
        depth = int(audio.get("bits_per_sample", 0))
        if depth == 0:
            depth = int(audio.get("bits_per_raw_sample", 32))
        bit_depth_list.append(depth)

    if not samplerate_list.count(samplerate_list[0]) == len(samplerate_list):
        print_exit("sample_mismatch", samplerate_list)
    if not channels_list.count(channels_list[0]) == len(channels_list):
        print_exit("channel_mismatch", channels_list)
    if not bit_depth_list.count(bit_depth_list[0]) == len(bit_depth_list):
        print_exit("bitdepth_mismatch", bit_depth_list)

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

    if channels not in [1, 2, 6, 8]:
        print_exit("channels")
    if downmix and downmix >= channels:
        print_exit("downmix_mismatch")
    if not downmix and aformat == "dd" and channels == 8:
        downmix = 6
    if aformat == "ac4" and channels != 6:
        print_exit("ac4_input_channels")
    if aformat == "ac4":
        downmix = 2
    if aformat == "thd" and channels == 1:
        print_exit("thd_mono_input")

    downmix_config = "off"
    if downmix:
        outchannels = downmix
        downmix_config = channel_number_to_name(outchannels)
    else:
        outchannels = channels

    if outchannels in [1, 2] and aformat in ["dd", "ddp"]:
        if payload.no_prompt:
            print(
                "Consider using [bold cyan]qaac[/bold cyan] or [bold cyan]opus[/bold cyan] for \
[bold yellow]mono[/bold yellow] and [bold yellow]stereo[/bold yellow] encoding."
            )
        else:
            continue_enc = Confirm.ask(
                "Consider using [bold cyan]qaac[/bold cyan] or [bold cyan]opus[/bold cyan] for \
[bold yellow]mono[/bold yellow] and [bold yellow]stereo[/bold yellow] encoding, are you sure you want to use [bold cyan]DEE[/bold cyan]?"
            )
            if not continue_enc:
                sys.exit(1)

    if outchannels == 2 and aformat == "thd":
        if payload.no_prompt:
            print(
                "Consider using [bold cyan]FLAC[/bold cyan] for lossless \
[bold yellow]mono[/bold yellow] and [bold yellow]stereo[/bold yellow] encoding."
            )
        else:
            continue_enc = Confirm.ask(
                "Consider using [bold cyan]FLAC[/bold cyan] for lossless \
[bold yellow]mono[/bold yellow] and [bold yellow]stereo[/bold yellow] encoding, are you sure you want to use [bold cyan]DEE[/bold cyan]?"
            )
            if not continue_enc:
                sys.exit(1)

    if payload.dialnorm != 0:
        if payload.no_prompt:
            print(
                "Consider leaving the dialnorm value at 0 (auto), setting it manually can be dangerous."
            )
        else:
            continue_enc = Confirm.ask(
                "Consider leaving the dialnorm value at 0 (auto), setting it manually can be dangerous, are you sure you want to do it?"
            )
            if not continue_enc:
                sys.exit(1)

    if aformat == "dd":
        if outchannels == 1:
            if not bitrate:
                bitrate = config["default_bitrates"]["dd_1_0"]
            bitrate = find_closest_allowed(bitrate, allowed_bitrates["dd_10"])
        elif outchannels == 2:
            if not bitrate:
                bitrate = config["default_bitrates"]["dd_2_0"]
            bitrate = find_closest_allowed(bitrate, allowed_bitrates["dd_20"])
        elif outchannels == 6:
            if not bitrate:
                bitrate = config["default_bitrates"]["dd_5_1"]
            bitrate = find_closest_allowed(bitrate, allowed_bitrates["dd_51"])
    if aformat == "ddp":
        if outchannels == 1:
            if not bitrate:
                bitrate = config["default_bitrates"]["ddp_1_0"]
            bitrate = find_closest_allowed(bitrate, allowed_bitrates["ddp_10"])
        elif outchannels == 2:
            if not bitrate:
                bitrate = config["default_bitrates"]["ddp_2_0"]
            bitrate = find_closest_allowed(bitrate, allowed_bitrates["ddp_20"])
        elif outchannels == 6:
            if not bitrate:
                bitrate = config["default_bitrates"]["ddp_5_1"]
            bitrate = find_closest_allowed(bitrate, allowed_bitrates["ddp_51"])
        elif outchannels == 8:
            if not bitrate:
                bitrate = config["default_bitrates"]["ddp_7_1"]
            if payload.force_standard:
                bitrate = find_closest_allowed(
                    bitrate, allowed_bitrates["ddp_71_standard"]
                )
            elif payload.force_bluray:
                bitrate = find_closest_allowed(
                    bitrate, allowed_bitrates["ddp_71_bluray"]
                )
            else:
                bitrate = find_closest_allowed(
                    bitrate, allowed_bitrates["ddp_71_combined"]
                )
    elif aformat == "ac4":
        if not bitrate:
            bitrate = config["default_bitrates"]["ac4_2_0"]
        bitrate = find_closest_allowed(bitrate, allowed_bitrates["ac4_20"])

    if payload.output_dir:
        createdir(os.path.abspath(payload.output_dir))
        output = os.path.abspath(payload.output_dir)
    else:
        output = os.getcwd()

    if aformat in ["dd", "ddp"]:
        xml_base = xmltodict.parse(xml_dd_ddp_base)
        xml_base["job_config"]["output"]["ec3"]["storage"]["local"]["path"] = wpc(
            output, quote=True
        )
        if aformat == "ddp":
            xml_base["job_config"]["filter"]["audio"]["pcm_to_ddp"][
                "encoder_mode"
            ] = "ddp"
            if outchannels == 8:
                xml_base["job_config"]["filter"]["audio"]["pcm_to_ddp"][
                    "encoder_mode"
                ] = "ddp71"
                if bitrate > 1024:
                    xml_base["job_config"]["filter"]["audio"]["pcm_to_ddp"][
                        "encoder_mode"
                    ] = "bluray"
                if payload.force_standard:
                    xml_base["job_config"]["filter"]["audio"]["pcm_to_ddp"][
                        "encoder_mode"
                    ] = "ddp71"
                if payload.force_bluray:
                    xml_base["job_config"]["filter"]["audio"]["pcm_to_ddp"][
                        "encoder_mode"
                    ] = "bluray"
        if aformat == "dd":
            xml_base["job_config"]["filter"]["audio"]["pcm_to_ddp"][
                "encoder_mode"
            ] = "dd"
        xml_base["job_config"]["filter"]["audio"]["pcm_to_ddp"][
            "downmix_config"
        ] = downmix_config
        xml_base["job_config"]["filter"]["audio"]["pcm_to_ddp"]["data_rate"] = bitrate
        xml_base["job_config"]["filter"]["audio"]["pcm_to_ddp"]["drc"][
            "line_mode_drc_profile"
        ] = payload.drc
        xml_base["job_config"]["filter"]["audio"]["pcm_to_ddp"]["drc"][
            "rf_mode_drc_profile"
        ] = payload.drc
        xml_base["job_config"]["filter"]["audio"]["pcm_to_ddp"][
            "custom_dialnorm"
        ] = payload.dialnorm
    elif aformat in ["ac4"]:
        xml_base = xmltodict.parse(xml_ac4_base)
        xml_base["job_config"]["output"]["ac4"]["storage"]["local"]["path"] = wpc(
            output, quote=True
        )
        xml_base["job_config"]["filter"]["audio"]["encode_to_ims_ac4"][
            "data_rate"
        ] = bitrate
        xml_base["job_config"]["filter"]["audio"]["encode_to_ims_ac4"]["drc"][
            "ddp_drc_profile"
        ] = payload.drc
        xml_base["job_config"]["filter"]["audio"]["encode_to_ims_ac4"]["drc"][
            "flat_panel_drc_profile"
        ] = payload.drc
        xml_base["job_config"]["filter"]["audio"]["encode_to_ims_ac4"]["drc"][
            "home_theatre_drc_profile"
        ] = payload.drc
        xml_base["job_config"]["filter"]["audio"]["encode_to_ims_ac4"]["drc"][
            "portable_hp_drc_profile"
        ] = payload.drc
        xml_base["job_config"]["filter"]["audio"]["encode_to_ims_ac4"]["drc"][
            "portable_spkr_drc_profile"
        ] = payload.drc
    elif aformat == "thd":
        xml_base = xmltodict.parse(xml_thd_base)
        xml_base["job_config"]["output"]["mlp"]["storage"]["local"]["path"] = wpc(
            output, quote=True
        )
        xml_base["job_config"]["filter"]["audio"]["encode_to_dthd"][
            "atmos_presentation"
        ]["drc_profile"] = payload.drc
        xml_base["job_config"]["filter"]["audio"]["encode_to_dthd"]["presentation_8ch"][
            "drc_profile"
        ] = payload.drc
        xml_base["job_config"]["filter"]["audio"]["encode_to_dthd"]["presentation_6ch"][
            "drc_profile"
        ] = payload.drc
        xml_base["job_config"]["filter"]["audio"]["encode_to_dthd"]["presentation_2ch"][
            "drc_profile"
        ] = payload.drc
        xml_base["job_config"]["filter"]["audio"]["encode_to_dthd"][
            "custom_dialnorm"
        ] = payload.dialnorm
    xml_base["job_config"]["input"]["audio"]["wav"]["storage"]["local"]["path"] = wpc(
        config["temp_path"], quote=True
    )
    xml_base["job_config"]["misc"]["temp_dir"]["path"] = wpc(
        config["temp_path"], quote=True
    )

    simplens.dee_version = parse_version_string([config["dee_path"]])
    simplens.ffmpeg_version = parse_version_string([config["ffmpeg_path"], "-version"])
    simplens.ffprobe_version = parse_version_string(
        [config["ffprobe_path"], "-version"]
    )

    if not all(x is False for x in config["summary_sections"].values()):
        summary = Table(
            title="Encoding summary",
            title_style="not italic bold magenta",
            show_header=False,
        )
        summary.add_column(style="green")
        summary.add_column(style="color(231)")

        if config["summary_sections"]["deew_info"]:
            try:
                r = requests.get(
                    "https://api.github.com/repos/pcroland/deew/releases/latest"
                )
                latest_version = json.loads(r.text)["tag_name"]
                if version.parse(ProgramInfo.prog_version) < version.parse(
                    latest_version
                ):
                    latest_version = f"[bold green]{latest_version}[/bold green] !!!"
            except Exception:
                latest_version = "[red]couldn't fetch"
            summary.add_row("[bold cyan]Version", ProgramInfo.prog_version)
            summary.add_row("[bold cyan]Latest", latest_version, end_section=True)

        if config["summary_sections"]["binaries"]:
            summary.add_row("[cyan]DEE version", simplens.dee_version)
            summary.add_row("[cyan]ffmpeg version", simplens.ffmpeg_version)
            summary.add_row(
                "[cyan]ffprobe version", simplens.ffprobe_version, end_section=True
            )

        if config["summary_sections"]["input_info"]:
            summary.add_row("[bold yellow]Input")
            summary.add_row("Channels", channel_number_to_name(channels))
            summary.add_row("Bit depth", str(bit_depth), end_section=True)

        if config["summary_sections"]["output_info"]:
            summary.add_row("[bold yellow]Output")
            summary.add_row("Format", "TrueHD" if aformat == "thd" else aformat.upper())
            summary.add_row(
                "Channels",
                "immersive stereo"
                if aformat == "ac4"
                else channel_number_to_name(outchannels),
            )
            summary.add_row(
                "Bitrate", "N/A" if aformat == "thd" else f"{str(bitrate)} kbps"
            )
            summary.add_row(
                "Dialnorm",
                "auto (0)" if payload.dialnorm == 0 else f"{str(payload.dialnorm)} dB",
                end_section=True,
            )

        if config["summary_sections"]["other"]:
            if payload.delay:
                delay_print, delay_xml, delay_mode = convert_delay_to_ms(
                    payload.delay, compensate=False
                )
            summary.add_row("[bold yellow]Other")
            summary.add_row("Files", str(len(filelist)))
            summary.add_row("Max instances", str(f"{instances:g}"))
            summary.add_row(
                "Delay",
                delay_print if payload.delay else "0 ms or parsed from filename",
            )
            summary.add_row("Temp path", config["temp_path"])

        print(summary)
        print()

    resample_value = ""
    if aformat in ["dd", "ddp", "ac4"] and samplerate != 48000:
        bit_depth = 32
        resample_value = "48000"
    elif aformat == "thd" and samplerate not in [48000, 96000]:
        bit_depth = 32
        if samplerate < 72000:
            resample_value = "48000"
        else:
            resample_value = "96000"
    if resample_value:
        if channels == 8:
            channel_swap = "pan=7.1|c0=c0|c1=c1|c2=c2|c3=c3|c4=c6|c5=c7|c6=c4|c7=c5,"
        else:
            channel_swap = ""
        resample_args = [
            "-filter_complex",
            f"[a:{trackindex}]{channel_swap}aresample=resampler=soxr",
            "-ar",
            resample_value,
            "-precision",
            "28",
            "-cutoff",
            "1",
            "-dither_scale",
            "0",
        ]
        resample_args_print = f'-filter_complex [bold color(231)]"\[a:{trackindex}]{channel_swap}aresample=resampler=soxr"[/bold color(231)] \
-ar [bold color(231)]{resample_value}[/bold color(231)] \
-precision [bold color(231)]28[/bold color(231)] \
-cutoff [bold color(231)]1[/bold color(231)] \
-dither_scale [bold color(231)]0[/bold color(231)] '
    else:
        resample_args = []
        resample_args_print = ""

    if channels == 8 and not resample_args:
        channel_swap_args = [
            "-filter_complex",
            f"[a:{trackindex}]pan=7.1|c0=c0|c1=c1|c2=c2|c3=c3|c4=c6|c5=c7|c6=c4|c7=c5",
        ]
        channel_swap_args_print = f'-filter_complex [bold color(231)]"\[a:{trackindex}]pan=7.1|c0=c0|c1=c1|c2=c2|c3=c3|c4=c6|c5=c7|c6=c4|c7=c5"[/bold color(231)] '
    else:
        channel_swap_args = []
        channel_swap_args_print = ""

    if simplens.is_nonnative_exe:
        dee_xml_input_base = wpc(config["temp_path"])
    else:
        dee_xml_input_base = (
            config["temp_path"]
            if config["temp_path"].endswith("/")
            else f'{config["temp_path"]}/'
        )

    xml_validation = [] if simplens.dee_is_exe else ["--disable-xml-validation"]
    xml_validation_print = "" if simplens.dee_is_exe else " --disable-xml-validation"

    if "-filter_complex" in resample_args or "-filter_complex" in channel_swap_args:
        map_args = []
        map_args_print = ""
    else:
        map_args = ["-map", f"0:a:{trackindex}"]
        map_args_print = (
            "-map [bold color(231)]0:a[/bold color(231)]"
            + f"[bold color(231)]:{trackindex}[/bold color(231)] "
        )

    settings = []
    ffmpeg_print_list = []
    dee_print_list = []
    intermediate_exists_list = []
    for i in range(len(filelist)):
        dee_xml_input = (
            f'{dee_xml_input_base}{basename(filelist[i], "xml", sanitize=True)}'
        )

        ffmpeg_args = [
            config["ffmpeg_path"],
            "-y",
            "-drc_scale",
            "0",
            "-i",
            filelist[i],
            *(map_args),
            "-c",
            f"pcm_s{bit_depth}le",
            *(channel_swap_args),
            *(resample_args),
            "-rf64",
            "always",
            os.path.join(config["temp_path"], basename(filelist[i], "wav")),
        ]
        dee_args = [
            config["dee_path"],
            "--progress-interval",
            "500",
            "--diagnostics-interval",
            "90000",
            "-x",
            dee_xml_input,
            *(xml_validation),
        ]

        ffmpeg_args_print = f'[bold cyan]ffmpeg[/bold cyan] \
-y \
-drc_scale [bold color(231)]0[/bold color(231)] \
-i [bold green]{filelist[i]}[/bold green] \
{map_args_print}\
-c [bold color(231)]pcm_s{bit_depth}le[/bold color(231)] \
{channel_swap_args_print}{resample_args_print}\
-rf64 [bold color(231)]always[/bold color(231)] \
[bold magenta]{os.path.join(config["temp_path"], basename(filelist[i], "wav"))}[/bold magenta]'

        ffmpeg_args_print_short = f"[bold cyan]ffmpeg[/bold cyan] \
-y \
-drc_scale [bold color(231)]0[/bold color(231)] \
-i [bold green]\[input][/bold green] \
{map_args_print}\
-c [bold color(231)]pcm_s{bit_depth}le[/bold color(231)] \
{channel_swap_args_print}{resample_args_print}\
-rf64 [bold color(231)]always[/bold color(231)] \
[bold magenta]\[output][/bold magenta]"

        dee_args_print = f"[bold cyan]dee[/bold cyan] -x [bold magenta]{dee_xml_input}[/bold magenta]{xml_validation_print}"
        dee_args_print_short = f"[bold cyan]dee[/bold cyan] -x [bold magenta]\[input][/bold magenta]{xml_validation_print}"

        intermediate_exists = False
        if os.path.exists(
            os.path.join(config["temp_path"], basename(filelist[i], "wav"))
        ):
            intermediate_exists = True
            ffmpeg_args_print = "[green]Intermediate already exists[/green]"

        ffmpeg_print_list.append(ffmpeg_args_print)
        dee_print_list.append(dee_args_print)
        if intermediate_exists:
            intermediate_exists_list.append(filelist[i])

        delay_in_filename = re.match(r".+DELAY ([-|+]?[0-9]+m?s)\..+", filelist[i])
        if delay_in_filename:
            delay = delay_in_filename[1]
            if not delay.startswith(("-", "+")):
                delay = f"+{delay}"
        else:
            delay = "+0ms"
        if payload.delay:
            delay = payload.delay

        xml = deepcopy(xml_base)
        xml["job_config"]["input"]["audio"]["wav"]["file_name"] = basename(
            filelist[i], "wav", quote=True
        )
        if aformat == "ddp":
            xml["job_config"]["output"]["ec3"]["file_name"] = basename(
                filelist[i], "ec3", quote=True, stripdelay=True
            )
            if bitrate > 1024:
                xml["job_config"]["output"]["ec3"]["file_name"] = basename(
                    filelist[i], "eb3", quote=True, stripdelay=True
                )
            if payload.force_standard:
                xml["job_config"]["output"]["ec3"]["file_name"] = basename(
                    filelist[i], "ec3", quote=True, stripdelay=True
                )
            if payload.force_bluray:
                xml["job_config"]["output"]["ec3"]["file_name"] = basename(
                    filelist[i], "eb3", quote=True, stripdelay=True
                )
        elif aformat == "dd":
            xml["job_config"]["output"]["ec3"]["file_name"] = basename(
                filelist[i], "ac3", quote=True, stripdelay=True
            )
            xml["job_config"]["output"]["ac3"] = xml["job_config"]["output"]["ec3"]
            del xml["job_config"]["output"]["ec3"]
        elif aformat == "ac4":
            xml["job_config"]["output"]["ac4"]["file_name"] = basename(
                filelist[i], "ac4", quote=True, stripdelay=True
            )
        else:
            xml["job_config"]["output"]["mlp"]["file_name"] = basename(
                filelist[i], "thd", quote=True, stripdelay=True
            )

        if aformat in ["dd", "ddp"]:
            delay_print, delay_xml, delay_mode = convert_delay_to_ms(
                delay, compensate=True
            )
            xml["job_config"]["filter"]["audio"]["pcm_to_ddp"][delay_mode] = delay_xml
        elif aformat in ["ac4"]:
            delay_print, delay_xml, delay_mode = convert_delay_to_ms(
                delay, compensate=True
            )
            xml["job_config"]["filter"]["audio"]["encode_to_ims_ac4"][
                delay_mode
            ] = delay_xml
        else:
            delay_print, delay_xml, delay_mode = convert_delay_to_ms(
                delay, compensate=False
            )
            xml["job_config"]["filter"]["audio"]["encode_to_dthd"][
                delay_mode
            ] = delay_xml

        save_xml(
            os.path.join(
                config["temp_path"], basename(filelist[i], "xml", sanitize=True)
            ),
            xml,
        )

        settings.append(
            [
                filelist[i],
                output,
                length_list[i],
                ffmpeg_args,
                dee_args,
                intermediate_exists,
                aformat,
            ]
        )

    if payload.long_argument:
        print("[bold color(231)]Running the following commands:[/bold color(231)]")
        for ff, d in zip(ffmpeg_print_list, dee_print_list):
            print(f"{ff} && {d}")
    else:
        if intermediate_exists_list:
            print(
                f'[bold color(231)]Intermediate already exists for the following file(s):[/bold color(231)] \
[bold magenta]{"[not bold white],[/not bold white] ".join(intermediate_exists_list)}[bold magenta]'
            )
        print(
            f"[bold color(231)]Running the following commands:[/bold color(231)]\n{ffmpeg_args_print_short} && {dee_args_print_short}"
        )

    print()

    pb = Progress(
        "[",
        "{task.description}",
        "]",
        BarColumn(),
        "[magenta]{task.percentage:>3.2f}%",
        refresh_per_second=8,
    )
    simplens.pb = pb

    signal.signal(signal.SIGINT, signal.SIG_DFL)

    jobs = []
    with pb:
        with ThreadPoolExecutor(max_workers=instances) as pool:
            for setting in settings:
                task_id = pb.add_task("", visible=False, total=None)
                jobs.append(pool.submit(encode, task_id, setting, payload, simplens))
    for job in jobs:
        job.result()
