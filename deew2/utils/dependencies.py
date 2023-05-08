from pathlib import Path
import shutil
from deeaw2.exceptions import DependencyNotFoundError


class Dependencies:
    ffmpeg = None
    dee = None


# TODO Re-enable some sort of config control, not sure
# how we want to do this yet.
# TODO make this better, i'm sure it can be improved to better dynamically handle
# dependencies.
class FindDependencies:
    """
    A utility class for finding and verifying dependencies required by a program.
    It first tries to locate the dependencies beside the program,
    then in the configuration file, and finally on the system PATH.

    Attributes:
        ffmpeg (str): The path to the FFmpeg executable, or None if not found.
        dee (str): The path to the Dee executable, or None if not found.

    Args:
        base_wd (Path): The base working directory of the program.
    """

    def get_dependencies(self, base_wd: Path):
        ffmpeg, dee = self._locate_beside_program(base_wd)

        # TODO re-implement this
        # if None in [self.ffmpeg, self.mkvextract, self.dee, self.gst_launch]:
        #     _create_config()
        #     self._locate_in_config()

        if None in [ffmpeg, dee]:
            self._locate_on_path()

        self._verify_dependencies([ffmpeg, dee])

        dependencies = Dependencies()
        dependencies.ffmpeg = ffmpeg
        dependencies.dee = dee

        return dependencies

    @staticmethod
    def _locate_beside_program(base_wd):
        ffmpeg_path = Path(base_wd / "apps/ffmpeg/ffmpeg.exe")
        dee_path = Path(base_wd / "apps/dee/dee.exe")

        found_paths = [str(path) for path in [ffmpeg_path, dee_path] if path.exists()]

        for path in found_paths:
            if str(path) == str(ffmpeg_path):
                ffmpeg_path = str(path)
            elif str(path) == str(dee_path):
                dee_path = str(path)

        return ffmpeg_path, dee_path

    # def _locate_in_config(self):
    #     attribute_names = ["ffmpeg", "dee"]
    #     config_section = "tool_paths"
    #     for attr_name in attribute_names:
    #         value = _read_config(config_section, attr_name)
    #         if value and Path(value).is_file():
    #             setattr(self, attr_name, str(value))

    @staticmethod
    def _locate_on_path():
        if ffmpeg is None:
            ffmpeg = shutil.which("ffmpeg")
        if dee is None:
            dee = shutil.which("dee")

        return ffmpeg, dee

    @staticmethod
    def _verify_dependencies(dependencies: list):
        executable_names = ["ffmpeg", "dee"]
        for exe_path, exe_name in zip(dependencies, executable_names):
            if exe_path is None or exe_path == "" or not Path(exe_path).is_file():
                raise DependencyNotFoundError(f"{exe_name} path not found")
