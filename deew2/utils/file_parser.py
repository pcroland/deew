from pathlib import Path
import glob


class FileParser:
    @staticmethod
    def parse_input_s(args_list: list):
        """
        Parse the input arguments and return a list of Path objects representing the input files.

        Args:
            args_list (list): List of input arguments.

        Returns:
            list: List of Path objects representing the input files.

        Raises:
            FileNotFoundError: If an input path is not a valid file path.
        """
        input_s = []
        for arg_input in args_list:
            # non recursive
            if "*" in arg_input:
                input_s.extend(Path(p) for p in glob.glob(arg_input))

            # recursive search
            elif "**" in arg_input:
                input_s.extend(Path(p) for p in glob.glob(arg_input, recursive=True))

            # single file path
            elif (
                Path(arg_input).exists()
                and Path(arg_input).is_file()
                and arg_input.strip() != ""
            ):
                input_s.append(Path(arg_input))
            else:
                raise FileNotFoundError(f"{arg_input} is not a valid input path.")

        return input_s
