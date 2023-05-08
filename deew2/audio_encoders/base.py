from pathlib import Path
import shutil
from deew2.exceptions import (
    ChannelMixError,
    InputFileNotFoundError,
    NotEnoughSpaceError,
)


class BaseAudioEncoder:
    @staticmethod
    def _check_for_up_mixing(source_channels: int, desired_channels: int):
        """Provide source_channels and ensure that desired channels is less than source"""
        if source_channels < desired_channels:
            raise ChannelMixError("Up-mixing is not supported.")

    @staticmethod
    def _check_input_file(input_file: Path):
        """Checks to ensure input file exists retuning a boolean value

        Args:
            input_file (Path): Input file path

        Returns:
            bool: True or False
        """
        if not input_file.exists():
            raise InputFileNotFoundError(f"Could not find {input_file.name}.")
        return input_file.exists()

    @staticmethod
    def _check_disk_space(drive_path: Path, required_space: int):
        """
        Check for free space at the drive path, rounding to nearest whole number.
        If there isn't at least "required_space" GB of space free, raise an ArgumentTypeError.

        Args:
            drive_path (Path): Path to check
            required_space (int): Minimum space (GB)
        """

        # get free space in bytes
        required_space_cwd = shutil.disk_usage(Path(drive_path)).free

        # convert to GB's
        free_space_gb = round(required_space_cwd / (1024**3))

        # check to ensure the desired space in GB's is free
        if free_space_gb < int(required_space):
            raise NotEnoughSpaceError(f"Insufficient storage to complete the process.")
        else:
            return True

    @staticmethod
    def _get_closest_allowed_bitrate(bitrate: int, accepted_bitrates: list):
        """Returns the closest allowed bitrate from a given input bitrate in a list of accepted bitrates.

        Args:
            bitrate (int): The input bitrate to find the closest allowed bitrate for.
            accepted_bitrates (list): A list of accepted bitrates.

        Returns:
            int: The closest allowed bitrate in the list of accepted bitrates.
        """
        return min(accepted_bitrates, key=lambda x: abs(x - bitrate))
