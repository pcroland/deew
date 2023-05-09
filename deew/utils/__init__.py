from types import SimpleNamespace
from cli.utils import print_exit
import os
import re
from unidecode import unidecode
from datetime import timedelta

from deew.simplens import simplens


def clamp(inp: int, low: int, high: int) -> int:
    return min(max(inp, low), high)


def trim_names(fl: str, compensate: int) -> str:
    if len(fl) > 40 - compensate:
        fl = f"{fl[0:37 - compensate]}..."
    return fl.ljust(40 - compensate, " ")


def stamp_to_sec(stamp):
    l = stamp.split(":")
    return int(l[0]) * 3600 + int(l[1]) * 60 + float(l[2])


def convert_delay_to_ms(inp, compensate):
    if not inp.startswith(("-", "+")):
        print_exit("delay")
    inp = inp.replace(",", ".")

    negative = inp.startswith(("-"))

    if "@" in inp:
        frame = round(float(re.sub("[^0-9\.]", "", inp.split("@")[0])))
        if not frame:
            print_exit("delay")

        fps = str(inp.split("@")[1])
        if fps == "ntsc":
            fps = str(24000 / 1001)
        if fps == "pal":
            fps = str(25)
        if "/" in fps:
            divident = re.sub("[^0-9\.]", "", fps.split("/")[0])
            divisor = re.sub("[^0-9\.]", "", fps.split("/")[1])
            if not divident or not divisor:
                print_exit("delay")
            fps = float(divident) / float(divisor)
        delay = frame / float(fps) * 1000
    else:
        if not inp.endswith(("s", "ms")):
            print_exit("delay")
        if not inp.count(".") < 2:
            print_exit("delay")

        if inp.endswith("ms"):
            delay = float(re.sub("[^0-9\.]", "", inp))
        else:
            delay = float(re.sub("[^0-9\.]", "", inp)) * 1000

    delay = float(f"-{delay}" if negative else f"+{delay}")
    delay_print = f'{format(delay, ".3f")} ms'

    if compensate:
        delay -= 16 / 3  # 256 / 48000 * 1000

    delay_mode = "prepend_silence_duration"
    if delay < 0:
        delay_mode = "start"
        delay_xml = str(timedelta(seconds=(abs(delay) / 1000)))
        if "." not in delay_xml:
            delay_xml = f"{delay_xml}.0"
    else:
        delay_xml = format(delay / 1000, ".6f")

    return delay_print, delay_xml, delay_mode


def channel_number_to_name(inp: int):
    channel_names = {1: "mono", 2: "stereo", 6: "5.1", 8: "7.1"}
    return channel_names[inp]


def find_closest_allowed(value: int, allowed_values: list[int]) -> int:
    return min(allowed_values, key=lambda list_value: abs(list_value - value))


def wpc(p: str, quote: bool = False) -> str:
    if not simplens.is_nonnative_exe:
        if quote:
            p = f'"{p}"'
        return p
    if not p.startswith("/mnt/"):
        print_exit("wsl_path", p)
    parts = list(filter(None, p.split("/")))[1:]
    parts[0] = parts[0].upper() + ":"
    p = "\\".join(parts) + "\\"
    if quote:
        p = f'"{p}"'
    return p


def rwpc(p: str) -> str:
    return re.sub(
        r"^([a-z]):/",
        lambda m: f"/mnt/{m.group(1).lower()}/",
        p.replace("\\", "/"),
        flags=re.IGNORECASE,
    )


def basename(
    fl: str,
    format_: str,
    quote: bool = False,
    sanitize: bool = False,
    stripdelay: bool = False,
) -> str:
    name = os.path.basename(os.path.splitext(fl)[0]) + f".{format_}"
    if stripdelay:
        name = re.sub(r" ?DELAY [-|+]?[0-9]+m?s", "", name)
    if sanitize:
        name = unidecode(name).replace(" ", "_")
    if quote:
        name = f'"{name}"'
    return name
