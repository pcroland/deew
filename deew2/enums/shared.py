from enum import Enum
from dataclasses import dataclass


class ProgressMode(Enum):
    STANDARD = 0
    DEBUG = 1
    SILENT = 3


class StereoDownmix(Enum):
    STANDARD = 0
    DPLII = 1


class DeeDelayModes(Enum):
    NEGATIVE = "start"
    POSITIVE = "prepend_silence_duration"


@dataclass
class DeeDelay:
    MODE: DeeDelayModes
    DELAY: str


class DeeFPS(Enum):
    FPS_NOT_INDICATED = "not_indicated"
    FPS_23_976 = "23.976"
    FPS_24 = "24"
    FPS_25 = "25"
    FPS_29_97 = "29.97"
    FPS_30 = "30"
    FPS_48 = "48"
    FPS_50 = "50"
    FPS_59_94 = "59.94"
    FPS_60 = "60"


class DeeDRC(Enum):
    FILM_STANDARD = "film_standard"
    FILM_LIGHT = "film_light"
    MUSIC_STANDARD = "music_standard"
    MUSIC_LIGHT = "music_light"
    SPEECH = "speech"
    OFF = "none"
