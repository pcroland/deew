import argparse
import re
from cli._version import ProgramInfo
from rich import print


class RParse(argparse.ArgumentParser):
    def _print_message(self, message, file=None):
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
