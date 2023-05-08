class NotEnoughSpaceError(Exception):
    """Custom error class to for insufficient storage"""


class PathTooLongError(Exception):
    """Custom error class to for path names that are too long"""


class InvalidExtensionError(Exception):
    """Custom error class for invalid file extensions"""


class ChannelMixError(Exception):
    """Custom error class for invalid channel mix configurations"""


class InputFileNotFoundError(Exception):
    """Custom error class for missing input files"""


class OutputFileNotFoundError(Exception):
    """Custom error class for missing input files"""


class MediaInfoError(Exception):
    """Custom class for MediaInfo errors"""


class DependencyNotFoundError(Exception):
    """Custom exception class to call when a dependency is not found"""


class XMLFileNotFoundError(Exception):
    """Custom class to return if XML file output was not found"""


class InvalidDelayError(Exception):
    """Class to raise in the event of an invalid delay input"""
