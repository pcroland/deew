from deew2.track_info.audio_track_info import AudioTrackInfo
from deew2.exceptions import MediaInfoError
from pymediainfo import MediaInfo
from pathlib import Path
from re import search


class AutoFileName:
    def generate_output_filename(
        self, media_info: MediaInfo, file_input: Path, track_index: int
    ):
        """Automatically generate an output file name

        Args:
            media_info (MediaInfo): pymediainfo object of input file
            file_input (Path): Path to input file
            track_index (int): Track index from args

        Returns:
            Path: Path of a automatically generated filename
        """
        # placeholder extension
        extension = ".tmp"

        # base directory/name
        base_dir = Path(file_input).parent
        base_name = Path(Path(file_input).name).with_suffix("")

        # if track index is 0 we can assume this audio is in a raw format
        if track_index == 0:
            file_name = f"{base_name}{extension}"
            return Path(base_dir / Path(file_name))

        # if track index is equal to or greater than 1, we can assume it's likely in a container of some
        # sort, so we'll go ahead and attempt to detect delay/language to inject into the title.
        elif track_index >= 1:
            delay = self._delay_detection(media_info, file_input, track_index)
            language = self._language_detection(media_info, track_index)
            file_name = f"{base_name}_{language}_{delay}{extension}"
            return Path(base_dir / Path(file_name))

    @staticmethod
    def _delay_detection(media_info: MediaInfo, file_input: Path, track_index: int):
        """Detect delay relative to video to inject into filename

        Args:
            media_info (MediaInfo): pymediainfo object of input file
            file_input (Path): Path to input file
            track_index (int): Track index from args

        Returns:
            str: Returns a formatted delay string
        """
        audio_track = media_info.tracks[track_index + 1]
        if Path(file_input).suffix == ".mp4":
            if audio_track.source_delay:
                delay_string = f"[delay {str(audio_track.source_delay)}ms]"
            else:
                delay_string = str("[delay 0ms]")
        else:
            if audio_track.delay_relative_to_video:
                delay_string = f"[delay {str(audio_track.delay_relative_to_video)}ms]"
            else:
                delay_string = str("[delay 0ms]")
        return delay_string

    @staticmethod
    def _language_detection(media_info: MediaInfo, track_index: int):
        """
        Detect language of input track, returning language in the format of
        "eng" instead of "en" or "english."

        Args:
            media_info (MediaInfo): pymediainfo object of input file
            track_index (int): Track index from args

        Returns:
            str: Returns a formatted language string
        """
        audio_track = media_info.tracks[track_index + 1]
        if audio_track.other_language:
            l_lengths = [len(lang) for lang in audio_track.other_language]
            l_index = next(
                (i for i, length in enumerate(l_lengths) if length == 3), None
            )
            language_string = (
                f"[{audio_track.other_language[l_index]}]"
                if l_index is not None
                else "[und]"
            )
        else:
            language_string = "[und]"
        return language_string


class MediainfoParser:
    def get_track_by_id(self, file_input: Path, track_index: int):
        """Returns an AudioTrackInfo object with metadata for the audio track at the specified index in the input file.

        Parameters:
            file_input (Path): The input file to extract audio track metadata from.
            track_index (int): The index of the audio track to extract metadata for.

        Returns:
            AudioTrackInfo: An object with the extracted audio track metadata, including fps, duration, sample rate, bit depth, and channels.

        Raises:
            MediaInfoError: If the specified track index is out of range or the specified track is not an audio track.
        """
        # parse the input file with MediaInfo lib
        mi_object = MediaInfo.parse(file_input)

        # verify
        self._verify_track_index(mi_object, track_index)
        self._verify_audio_track(mi_object, track_index)

        # initiate AudioTrackInfo class
        audio_info = AudioTrackInfo()

        # update AudioTrackInfo with needed values
        audio_info.fps = self._get_fps(mi_object)
        audio_info.duration = self._get_duration(mi_object, track_index)
        audio_info.sample_rate = mi_object.tracks[track_index + 1].sampling_rate
        audio_info.bit_depth = mi_object.tracks[track_index + 1].bit_depth
        audio_info.channels = self._get_channels(mi_object, track_index)
        audio_info.auto_name = AutoFileName().generate_output_filename(
            mi_object, file_input, track_index
        )

        # return object
        return audio_info

    @staticmethod
    def _verify_track_index(mi_object, track_index):
        """
        Verify that the requested track exists in the MediaInfo object.

        Args:
            mi_object (MediaInfo): A MediaInfo object containing information about a media file.
            track_index (int): The index of the requested track.

        Raises:
            MediaInfoError: If the requested track does not exist in the MediaInfo object.
        """
        try:
            mi_object.tracks[track_index + 1]
        except IndexError:
            raise MediaInfoError(f"Selected track #{track_index} does not exist.")

    @staticmethod
    def _verify_audio_track(mi_object, track_index):
        """
        Checks that the specified track index in the given MediaInfo object corresponds to an audio track.

        Args:
            mi_object: A MediaInfo object.
            track_index: An integer representing the index of the track to be verified.

        Raises:
            MediaInfoError: If the specified track index does not correspond to an audio track.
        """
        track_info = mi_object.tracks[track_index + 1].track_type
        if track_info != "Audio":
            raise MediaInfoError(
                f"Selected track #{track_index} ({track_info}) is not an audio track."
            )

    @staticmethod
    def _get_fps(mi_object):
        """
        Get the frames per second (fps) for the video track in the media info object.

        Args:
            mi_object (MediaInfo): A MediaInfo object.

        Returns:
            fps (float or None): The frames per second (fps) for the video track, or None if there is no video track.
        """
        for mi_track in mi_object.tracks:
            if mi_track.track_type == "Video":
                fps = mi_track.frame_rate
                break
            else:
                fps = None
        return fps

    @staticmethod
    def _get_duration(mi_object, track_index):
        """
        Retrieve the duration of a specified track in milliseconds.

        Parameters:
            mi_object (MediaInfoDLL.MediaInfo): A MediaInfo object containing information about a media file.
            track_index (int): The index of the track for which to retrieve the duration.

        Returns:
            duration (float or None): The duration of the specified track in milliseconds, or None if the duration cannot be retrieved.
        """
        duration = mi_object.tracks[track_index + 1].duration
        if duration:
            duration = float(duration)
        return duration

    @staticmethod
    def _get_channels(mi_object, track_index):
        """
        Get the number of audio channels for the specified track.

        Args:
            mi_object (MediaInfo): A MediaInfo object containing information about the media file.
            track_index (int): The index of the track to extract information from.

        Returns:
            The number of audio channels as an integer.
        """
        track = mi_object.tracks[track_index + 1]
        base_channels = track.channel_s
        check_other = search(r"\d+", str(track.other_channel_s[0]))
        if check_other:
            return max(int(base_channels), int(check_other.group()))
        else:
            return base_channels
