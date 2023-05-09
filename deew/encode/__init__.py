import subprocess
import re
import os
import time

from types import SimpleNamespace

from rich.progress import TaskID
from builtins import print as oprint

from packaging import version

from deew.utils import (
    basename,
    clamp,
    trim_names,
    stamp_to_sec,
)

from deew.payloads import DeePayload


def encode(
    task_id: TaskID, settings: list, payload: DeePayload, simplens: SimpleNamespace
) -> None:
    config, pb = simplens.config, simplens.pb
    (
        fl,
        output,
        length,
        ffmpeg_args,
        dee_args,
        intermediate_exists,
        aformat,
    ) = settings
    fl_b = os.path.basename(fl)
    pb.update(
        description=f'[bold][cyan]starting[/cyan][/bold]...{" " * 24}',
        task_id=task_id,
        visible=True,
    )

    if not intermediate_exists:
        if length == -1:
            pb.update(
                description=f"[bold][cyan]ffmpeg[/cyan][/bold] | {trim_names(fl_b, 6)}",
                task_id=task_id,
                total=None,
            )
            ffmpeg = subprocess.run(
                ffmpeg_args,
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
                universal_newlines=True,
                encoding="utf-8",
                errors="ignore",
            )
        else:
            pb.update(
                description=f"[bold][cyan]ffmpeg[/cyan][/bold] | {trim_names(fl_b, 6)}",
                task_id=task_id,
                total=100,
            )
            ffmpeg = subprocess.Popen(
                ffmpeg_args,
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
                universal_newlines=True,
                encoding="utf-8",
                errors="ignore",
            )
            percentage_length = length / 100
            with ffmpeg.stdout:
                for line in iter(ffmpeg.stdout.readline, ""):
                    if "=" not in line:
                        continue
                    progress = re.search(r"time=([^\s]+)", line)
                    if progress:
                        timecode = stamp_to_sec(progress[1]) / percentage_length
                        pb.update(task_id=task_id, completed=timecode)
        pb.update(task_id=task_id, completed=100)
        time.sleep(0.5)

    if payload.dialnorm != 0 and aformat == "thd":
        pb.update(
            description=f"[bold cyan]DEE[/bold cyan]: encode | {trim_names(fl_b, 11)}",
            task_id=task_id,
            completed=0,
            total=100,
        )
    else:
        pb.update(
            description=f"[bold cyan]DEE[/bold cyan]: measure | {trim_names(fl_b, 12)}",
            task_id=task_id,
            completed=0,
            total=100,
        )
    dee = subprocess.Popen(
        dee_args,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        universal_newlines=True,
        encoding="utf-8",
        errors="ignore",
    )
    with dee.stdout:
        encoding_step = False
        for line in iter(dee.stdout.readline, ""):
            if not encoding_step and re.search(
                r"(measured_loudness|speech gated loudness)", line
            ):
                encoding_step = True
                measured_dn = re.search(
                    r"(measured_loudness|speech\ gated\ loudness)(\=|\:\ )(-?.+)",
                    line,
                )
                measured_dn = round(float(measured_dn[3].strip(".")))
                measured_dn = str(clamp(measured_dn, -31, 0))
                if payload.measure_only:
                    pb.update(
                        description=f"[bold cyan]DEE[/bold cyan]: measure | {trim_names(fl_b, 18 + len(measured_dn))} ({measured_dn} dB)",
                        task_id=task_id,
                        completed=100,
                    )
                    dee.kill()
                else:
                    pb.update(
                        description=f"[bold cyan]DEE[/bold cyan]: encode | {trim_names(fl_b, 17 + len(measured_dn))} ({measured_dn} dB)",
                        task_id=task_id,
                    )

            progress = re.search(r"Stage progress: ([0-9]+\.[0-9])", line)
            if progress and progress[1] != "100.0":
                if (
                    aformat != "thd"
                    and version.parse(simplens.dee_version) >= version.parse("5.2.0")
                    and not encoding_step
                ):
                    pb.update(task_id=task_id, completed=float(progress[1]) / 4)
                else:
                    pb.update(task_id=task_id, completed=float(progress[1]))

            if "error" in line.lower():
                oprint(line.rstrip().split(": ", 1)[1])
    pb.update(task_id=task_id, completed=100)

    if not payload.keeptemp:
        os.remove(os.path.join(config["temp_path"], basename(fl, "wav")))
        os.remove(os.path.join(config["temp_path"], basename(fl, "xml", sanitize=True)))

    if payload.format.lower() == "thd":
        os.remove(os.path.join(output, basename(fl, "thd.log")))
        os.remove(os.path.join(output, basename(fl, "thd.mll")))
