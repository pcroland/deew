from typing import Union
from subprocess import Popen, PIPE, STDOUT
import re
from deeaw2.utils.utils import PrintSameLine
from deeaw2.enums.shared import ProgressMode


# TODO Modify this to work with more than just DEE, for now hard coded to DEE's uses
class ProcessFFMPEG:
    def process_job(
        self,
        cmd: list,
        progress_mode: ProgressMode,
        steps: bool,
        duration: Union[float, None],
    ):
        """Processes file with FFMPEG while generating progress depending on progress_mode.

        Args:
            cmd (list): Base FFMPEG command list
            progress_mode (ProgressMode): Options are ProgressMode.STANDARD or ProgressMode.DEBUG
            steps (bool): True or False, to disable updating encode steps
            duration (Union[float, None]): Can be None or duration in milliseconds
            If set to None the generic FFMPEG output will be displayed
            If duration is passed then we can calculate the total progress for FFMPEG
        """
        # inject verbosity level into cmd list depending on progress_mode
        inject = cmd.index("-v") + 1
        if progress_mode == ProgressMode.STANDARD:
            cmd.insert(inject, "quiet")
        elif progress_mode == ProgressMode.DEBUG:
            cmd.insert(inject, "info")

        with Popen(cmd, stdout=PIPE, stderr=STDOUT, universal_newlines=True) as proc:
            if progress_mode == "standard" and steps:
                print("---- Step 1 of 3 ---- [FFMPEG]")

            # initiate print on same line
            print_same_line = PrintSameLine()

            for line in proc.stdout:
                # Some audio formats actually do not have a "duration" in their raw containers,
                # if this is the case we will default ffmpeg to it's generic output string.
                if duration and progress_mode == ProgressMode.STANDARD:
                    # we need to wait for size= to prevent any errors
                    if "size=" in line:
                        percentage = self._convert_ffmpeg_to_percent(line, duration)

                        # update progress but break when 100% is met to prevent printing 100% multiple times
                        if percentage != "100.0%":
                            print_same_line.print_msg(percentage)
                        else:
                            print_same_line.print_msg("100.0%\n")
                            break
                else:
                    print(line.strip())

        if proc.returncode != 0:
            raise ValueError("There was an FFMPEG error. Please re-run in debug mode.")
        else:
            return True

    @staticmethod
    def _convert_ffmpeg_to_percent(line: str, duration: float):
        """
        Detect the format of 'HH:MM:SS' that FFMPEG provides, and convert it to milliseconds.
        This will allow us to generate an overall percentage based on the audio track's duration from the input.

        Args:
            line (str): FFMPEG generic output string
            duration (float): Source's audio track duration (ms)

        Returns:
            str: Formatted %
        """
        time = re.search(r"(\d\d):(\d\d):(\d\d)", line.strip())
        if time:
            total_ms = (
                int(time.group(1)) * 3600000
                + int(time.group(2)) * 60000
                + int(time.group(3)) * 1000
            )
            progress = float(total_ms) / float(duration)
            percent = "{:.1%}".format(min(1.0, progress))
            return percent
