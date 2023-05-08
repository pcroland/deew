import argparse
import re
import sys
import os
import requests
import json
import subprocess
from base64 import b64decode
from deew._version import ProgramInfo
from rich import print
from rich.prompt import Prompt
from rich.console import Console
from rich.syntax import Syntax
from deew.messages import error_messages
from deew.logos import logos
from deew.bitrates import allowed_bitrates
from typing import Any, NoReturn


class RParse(argparse.ArgumentParser):
    def _print_message(self, message, _):
        if message:
            if message.startswith("usage"):
                message = f"[bold cyan]{ProgramInfo.prog_name}[/bold cyan] {ProgramInfo.prog_version}\n\n{message}"
                message = re.sub(
                    r"(-[a-z]+\s*|\[)([A-Z]+)(?=]|,|\s\s|\s\.)",
                    r"\1[{}]\2[/{}]".format("bold color(231)", "bold color(231)"),
                    message,
                )
                message = re.sub(
                    r"((-|--)[a-z]+)", r"[{}]\1[/{}]".format("green", "green"), message
                )
                message = message.replace("usage", f"[yellow]USAGE[/yellow]")
                message = message.replace("options", f"[yellow]FLAGS[/yellow]", 1)
                message = message.replace(
                    self.prog, f"[bold cyan]{self.prog}[/bold cyan]"
                )
            message = f"[not bold white]{message.strip()}[/not bold white]"
            print(message)


class CustomHelpFormatter(argparse.RawTextHelpFormatter):
    def _format_action_invocation(self, action):
        if not action.option_strings or action.nargs == 0:
            return super()._format_action_invocation(action)
        default = self._get_default_metavar_for_optional(action)
        args_string = self._format_args(action, default)
        return ", ".join(action.option_strings) + " " + args_string


def print_exit(message: str, insert: Any = None) -> NoReturn:
    if insert and "🤠" in error_messages[message]:
        message_split = error_messages[message].split("🤠")
        before, after = message_split[0], message_split[1]
        exit_message = (
            f"[color(231) on red]ERROR:[/color(231) on red] {before}{insert}{after}"
        )
    else:
        exit_message = (
            f"[color(231) on red]ERROR:[/color(231) on red] {error_messages[message]}"
        )
    print(exit_message)
    sys.exit(1)


def print_logos() -> None:
    for i, logo in enumerate(logos):
        print(f"logo {i + 1}:\n{logo}")
    sys.exit(0)


def list_bitrates() -> None:
    for codec, bitrates in allowed_bitrates.items():
        print(
            f'[bold magenta]{codec}[/bold magenta]: [not bold color(231)]{"[white], [/white]".join([str(int) for int in bitrates])}[/not bold color(231)]'
        )
    sys.exit(0)


def parse_version_string(inp: list) -> str:
    try:
        v = subprocess.run(inp, capture_output=True, encoding="utf-8").stdout
        v = v.split("\n")[0].split(" ")[2]
        v = v.replace(",", "").replace("-static", "")
        if len(v) > 30:
            v = f"{v[0:27]}..."
    except Exception:
        v = "[red]couldn't parse"
    return v


def createdir(out: str) -> None:
    try:
        os.makedirs(out, exist_ok=True)
    except OSError:
        print_exit("create_dir", out)


def generate_config(standalone: bool, conf1: str, conf2: str, conf_dir: str) -> None:
    config_content = """# These are required.
    # If only name is specified, it will look in your system PATH variable, which includes the current directory on Windows.
    # Setup instructions: https://github.com/pcroland/deew#setup-system-path-variable
    # If full path is specified, that will be used.
    ffmpeg_path = 'ffmpeg'
    ffprobe_path = 'ffprobe'
    dee_path = 'dee.exe'

    # If this is empty, the default OS temporary directory will be used (or `temp` next to the script if you use the exe).
    # You can also specify an absolute path or a path relative to the current directory.
    temp_path = ''

    # Set between 1 and 10, use the -pl/--print-logos option to see the available logos, set to 0 to disable logo.
    logo = 1

    # Specifies how many encodes can run at the same time.
    # It can be a number or a % compared to your number of threads (so '50%' means 4 on an 8 thread cpu).
    # One DEE can use 2 threads so setting '50%' can utilize all threads.
    # You can override this setting with -in/--instances.
    # The number will be clamped between 1 and cpu_count().
    # With the Windows version of DEE the max will be cpu_count() - 2 or 6 due to a limitation.
    # examples: 1, 4, '50%'
    max_instances = '50%'

    [default_bitrates]
        dd_1_0 = 128
        dd_2_0 = 256
        dd_5_1 = 640
        ddp_1_0 = 128
        ddp_2_0 = 256
        ddp_5_1 = 1024
        ddp_7_1 = 1536
        ac4_2_0 = 320

    # You can toggle what sections you would like to see in the encoding summary
    [summary_sections]
        deew_info = true
        binaries = true
        input_info = true
        output_info = true
        other = true
    """

    if standalone:
        print(
            f"""Please choose config's location:
    [bold magenta]1[/bold magenta]: {conf1}
    [bold magenta]2[/bold magenta]: {conf2}"""
        )
        c_loc = Prompt.ask("Location", choices=["1", "2"])
        if c_loc == "1":
            createdir(conf_dir)
            c_loc = conf1
        else:
            c_loc = conf2
    else:
        c_loc = conf1
        createdir(conf_dir)

    with open(c_loc, "w") as fl:
        fl.write(config_content)
    print()
    Console().print(Syntax(config_content, "toml"))
    print(f"\n[bold cyan]The above config has been created at:[/bold cyan]\n{c_loc}")
    sys.exit(1)


def print_changelog() -> None:
    try:
        r = requests.get(
            "https://api.github.com/repos/pcroland/deew/contents/changelog.md"
        )
        changelog = json.loads(r.text)["content"]
        changelog = b64decode(changelog).decode().split("\n\n")
        changelog.reverse()
        changelog = "\n\n".join(changelog[-10:])
        changelog = changelog.split("\n")
    except Exception:
        print_exit("changelog")

    for line in changelog:
        if line.endswith("\\"):
            line = line[:-1]
        line = line.replace("\\", "\\\\")
        if line.startswith("# "):
            line = f'[bold color(231)]{line.replace("# ", "")}[/bold color(231)]'
        code_number = line.count("`")
        state_even = False
        for _ in range(code_number):
            if not state_even:
                line = line.replace("`", "[bold yellow]", 1)
                state_even = True
            else:
                line = line.replace("`", "[/bold yellow]", 1)
                state_even = False
        print(f"[not bold white]{line}[/not bold white]")
    sys.exit(0)
