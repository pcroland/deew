import xmltodict
from pathlib import Path
from typing import Union

from deew2.audio_encoders.dee.xml.dd_ddp_base_xml import xml_audio_base_ddp
from deew2.enums.shared import DeeFPS, DeeDelay, DeeDRC
from deew2.enums.dd import DolbyDigitalChannels
from deew2.enums.ddp import DolbyDigitalPlusChannels
from deew2.exceptions import XMLFileNotFoundError


class DeeXMLGenerator:
    """Handles the parsing/creation of XML file for DEE encoding"""

    def __init__(
        self,
        bitrate: str,
        wav_file_name: str,
        output_file_name: str,
        output_dir: Union[Path, str],
        fps: DeeFPS,
        delay: Union[DeeDelay, None],
        drc: DeeDRC,
    ):
        """Set shared values for both DD/DDP

        Args:
            bitrate (str): Bitrate in the format of '448'
            wav_file_name (str): File name only
            output_file_name (str): File name only
            output_dir (Union[Path, str]): File path only
            fps (DeeFPS): FPS of video input if it exists
            delay (Union[DeeDelay, None]): Dataclass holding DeeDelay values or None
            drc (DeeDRC): Dynamic range compression setting
        """
        # outputs
        self.output_file_name = output_file_name
        self.output_dir = output_dir

        # bitrate
        self.bitrate = bitrate

        # Parse base template
        self.xml_base = xmltodict.parse(xml_audio_base_ddp)

        # xml wav filename/path
        self.xml_base["job_config"]["input"]["audio"]["wav"][
            "file_name"
        ] = f'"{wav_file_name}"'
        self.xml_base["job_config"]["input"]["audio"]["wav"]["storage"]["local"][
            "path"
        ] = f'"{str(output_dir)}"'

        # xml output file/path
        self.xml_base["job_config"]["output"]["ac3"][
            "file_name"
        ] = f'"{output_file_name}"'
        self.xml_base["job_config"]["output"]["ac3"]["storage"]["local"][
            "path"
        ] = f'"{str(output_dir)}"'

        # update fps sections
        self.xml_base["job_config"]["input"]["audio"]["wav"][
            "timecode_frame_rate"
        ] = fps.value
        self.xml_base["job_config"]["filter"]["audio"]["pcm_to_ddp"][
            "timecode_frame_rate"
        ] = fps.value

        # xml temp path config
        self.xml_base["job_config"]["misc"]["temp_dir"]["path"] = f'"{str(output_dir)}"'

        # xml bit rate config
        self.xml_base["job_config"]["filter"]["audio"]["pcm_to_ddp"]["data_rate"] = str(
            bitrate
        )

        # xml delay config
        if delay:
            self.xml_base["job_config"]["filter"]["audio"]["pcm_to_ddp"][
                delay.MODE.value
            ] = delay.DELAY

        # xml dynamic range compression config
        # drc_path = self.xml_base["job_config"]["filter"]["audio"]["pcm_to_ddp"]["drc"]
        # drc_path["line_mode_drc_profile"] = drc.value
        # drc_path["rf_mode_drc_profile"] = drc.value

    def generate_xml_dd(
        self, down_mix_config: str, stereo_down_mix: str, channels: DolbyDigitalChannels
    ):
        """Generates an XML file for a Dolby Digital (DD) audio encoding job with the
        specified parameters and saves it to disk.

        Args:
            down_mix_config (str): The path to a downmix configuration file or None if no
                downmix is needed.
            stereo_down_mix (str): The preferred stereo downmix mode: 'standard' for Dolby
                Pro Logic II or 'dplii' for Lt/Rt.
            channels (DolbyDigitalChannels): The number of audio channels to encode. Can be
                one of DolbyDigitalChannels.MONO, DolbyDigitalChannels.STEREO,
                DolbyDigitalChannels.SURROUND, or
                DolbyDigitalChannels.SURROUNDEX.

        Returns:
            Path: The path to the generated XML file.
        """

        # xml down mix config
        if down_mix_config:
            self.xml_base["job_config"]["filter"]["audio"]["pcm_to_ddp"][
                "downmix_config"
            ] = down_mix_config
        else:
            del self.xml_base["job_config"]["filter"]["audio"]["pcm_to_ddp"][
                "downmix_config"
            ]

        # detect down_mix mode
        if channels == DolbyDigitalChannels.MONO:
            downmix_mode = "not_indicated"
        elif channels == DolbyDigitalChannels.STEREO:
            if stereo_down_mix == "standard":
                downmix_mode = "ltrt"
            elif stereo_down_mix == "dplii":
                downmix_mode = None
        elif channels == DolbyDigitalChannels.SURROUND:
            downmix_mode = "loro"

        # if downmix_mode is not None update the XML entry
        if downmix_mode:
            self.xml_base["job_config"]["filter"]["audio"]["pcm_to_ddp"]["downmix"][
                "preferred_downmix_mode"
            ] = downmix_mode

        # if not downmix_mode delete XML entry entirely
        else:
            del self.xml_base["job_config"]["filter"]["audio"]["pcm_to_ddp"]["downmix"][
                "preferred_downmix_mode"
            ]

        # xml encoder format
        self.xml_base["job_config"]["filter"]["audio"]["pcm_to_ddp"][
            "encoder_mode"
        ] = "dd"

        # save xml
        xml_file = self._save_xml(self.output_dir, self.output_file_name, self.xml_base)

        return xml_file

    def generate_xml_ddp(
        self,
        down_mix_config: str,
        stereo_down_mix: str,
        channels: DolbyDigitalChannels,
        normalize: bool,
    ):
        """Generates an XML file for a Dolby Digital Plus (DDP) audio encoding job with the
        specified parameters and saves it to disk.

        Args:
            down_mix_config (str): The path to a downmix configuration file or None if no
                downmix is needed.
            stereo_down_mix (str): The preferred stereo downmix mode: 'standard' for Dolby
                Pro Logic II or 'dplii' for Lt/Rt.
            channels (DolbyDigitalChannels): The number of audio channels to encode. Can be
                one of DolbyDigitalPlusChannels.MONO, DolbyDigitalPlusChannels.STEREO,
                DolbyDigitalPlusChannels.SURROUND, or
                DolbyDigitalPlusChannels.SURROUNDEX.
            normalize (bool): Whether to normalize the audio using the Dolby Dialogue
                Intelligence (DDI) algorithm.

        Returns:
            Path: The path to the generated XML file.
        """
        # xml down mix config
        if down_mix_config:
            self.xml_base["job_config"]["filter"]["audio"]["pcm_to_ddp"][
                "downmix_config"
            ] = down_mix_config
        else:
            del self.xml_base["job_config"]["filter"]["audio"]["pcm_to_ddp"][
                "downmix_config"
            ]

        # detect down_mix mode
        if channels in [
            DolbyDigitalPlusChannels.MONO,
            DolbyDigitalPlusChannels.SURROUNDEX,
        ]:
            downmix_mode = "not_indicated"
        elif channels == DolbyDigitalPlusChannels.STEREO:
            if stereo_down_mix == "standard":
                downmix_mode = "ltrt"
            elif stereo_down_mix == "dplii":
                downmix_mode = "ltrt-pl2"
        elif channels == DolbyDigitalPlusChannels.SURROUND:
            downmix_mode = "loro"

        # if downmix_mode is not None update the XML entry
        self.xml_base["job_config"]["filter"]["audio"]["pcm_to_ddp"]["downmix"][
            "preferred_downmix_mode"
        ] = downmix_mode

        # if ddp and normalize is true, set template to normalize audio
        if normalize:
            # TODO allow all supported presets later
            # Remove measure_only, add measure_and_correct, with default preset of atsc_a85
            del self.xml_base["job_config"]["filter"]["audio"]["pcm_to_ddp"][
                "loudness"
            ]["measure_only"]
            self.xml_base["job_config"]["filter"]["audio"]["pcm_to_ddp"]["loudness"][
                "measure_and_correct"
            ] = {}
            self.xml_base["job_config"]["filter"]["audio"]["pcm_to_ddp"]["loudness"][
                "measure_and_correct"
            ]["preset"] = "atsc_a85"

        # xml encoder format
        # if channels are 8 set encoder mode to ddp71
        if channels == DolbyDigitalPlusChannels.SURROUNDEX:
            # set encoder mode based on bitrate, under 1024 and under would be
            # standard (web)
            if int(self.bitrate) <= 1024:
                self.xml_base["job_config"]["filter"]["audio"]["pcm_to_ddp"][
                    "encoder_mode"
                ] = "ddp71"

            # over 1024 would be considered 'bluray'
            elif int(self.bitrate) > 1024:
                self.xml_base["job_config"]["filter"]["audio"]["pcm_to_ddp"][
                    "encoder_mode"
                ] = "bluray"

        # if channels are less than 8 set encoder to ddp
        else:
            self.xml_base["job_config"]["filter"]["audio"]["pcm_to_ddp"][
                "encoder_mode"
            ] = "ddp"

        # set output mode to ec3 instead of ac3
        self.xml_base["job_config"]["output"]["ec3"] = self.xml_base["job_config"][
            "output"
        ]["ac3"]

        # delete ac3 from dict
        del self.xml_base["job_config"]["output"]["ac3"]

        # save xml
        xml_file = self._save_xml(self.output_dir, self.output_file_name, self.xml_base)

        return xml_file

    @staticmethod
    def _save_xml(output_dir: Path, output_file_name: Path, xml_base: dict):
        """Creates/Deletes old XML files for use with DEE

        Args:
            output_dir (Path): Full output directory
            output_file_name (Path): File name
            xml_base (dict): XML generated dictionary

        Returns:
            Path: Path to XML file for DEE
        """
        # Save out the updated template (use filename output with xml suffix)
        updated_template_file = Path(output_dir / Path(output_file_name)).with_suffix(
            ".xml"
        )

        # delete xml output template if one already exists
        if updated_template_file.exists():
            updated_template_file.unlink()

        # write new xml template for dee
        with open(updated_template_file, "w", encoding="utf-8") as xml_out:
            xml_out.write(xmltodict.unparse(xml_base, pretty=True, indent="  "))

        # check to ensure template file was created
        if updated_template_file.exists():
            return updated_template_file
        else:
            raise XMLFileNotFoundError("XML file could not be created")
