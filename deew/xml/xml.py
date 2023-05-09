import xmltodict
from pathlib import Path
from typing import Union
from copy import deepcopy
import os
from deew.utils import wpc, basename


def xml_generation(
    aformat,
    xml_dd_ddp_base,
    output,
    outchannels,
    bitrate,
    payload,
    downmix_config,
    xml_ac4_base,
    xml_thd_base,
    config,
    filelist,
):
    """Temporary solution to all of the nested XML code in the main script"""
    # TODO break up all the XML code, right now it's way to nested in main function
    pass
    # if aformat in ["dd", "ddp"]:
    #     xml_base = xmltodict.parse(xml_dd_ddp_base)
    #     xml_base["job_config"]["output"]["ec3"]["storage"]["local"]["path"] = wpc(
    #         output, quote=True
    #     )
    #     if aformat == "ddp":
    #         xml_base["job_config"]["filter"]["audio"]["pcm_to_ddp"][
    #             "encoder_mode"
    #         ] = "ddp"
    #         if outchannels == 8:
    #             xml_base["job_config"]["filter"]["audio"]["pcm_to_ddp"][
    #                 "encoder_mode"
    #             ] = "ddp71"
    #             if bitrate > 1024:
    #                 xml_base["job_config"]["filter"]["audio"]["pcm_to_ddp"][
    #                     "encoder_mode"
    #                 ] = "bluray"
    #             if payload.force_standard:
    #                 xml_base["job_config"]["filter"]["audio"]["pcm_to_ddp"][
    #                     "encoder_mode"
    #                 ] = "ddp71"
    #             if payload.force_bluray:
    #                 xml_base["job_config"]["filter"]["audio"]["pcm_to_ddp"][
    #                     "encoder_mode"
    #                 ] = "bluray"
    #     if aformat == "dd":
    #         xml_base["job_config"]["filter"]["audio"]["pcm_to_ddp"][
    #             "encoder_mode"
    #         ] = "dd"
    #     xml_base["job_config"]["filter"]["audio"]["pcm_to_ddp"][
    #         "downmix_config"
    #     ] = downmix_config
    #     xml_base["job_config"]["filter"]["audio"]["pcm_to_ddp"][
    #         "data_rate"
    #     ] = bitrate
    #     xml_base["job_config"]["filter"]["audio"]["pcm_to_ddp"]["drc"][
    #         "line_mode_drc_profile"
    #     ] = payload.drc
    #     xml_base["job_config"]["filter"]["audio"]["pcm_to_ddp"]["drc"][
    #         "rf_mode_drc_profile"
    #     ] = payload.drc
    #     xml_base["job_config"]["filter"]["audio"]["pcm_to_ddp"][
    #         "custom_dialnorm"
    #     ] = payload.dialnorm
    # elif aformat in ["ac4"]:
    #     xml_base = xmltodict.parse(xml_ac4_base)
    #     xml_base["job_config"]["output"]["ac4"]["storage"]["local"]["path"] = wpc(
    #         output, quote=True
    #     )
    #     xml_base["job_config"]["filter"]["audio"]["encode_to_ims_ac4"][
    #         "data_rate"
    #     ] = bitrate
    #     xml_base["job_config"]["filter"]["audio"]["encode_to_ims_ac4"]["drc"][
    #         "ddp_drc_profile"
    #     ] = payload.drc
    #     xml_base["job_config"]["filter"]["audio"]["encode_to_ims_ac4"]["drc"][
    #         "flat_panel_drc_profile"
    #     ] = payload.drc
    #     xml_base["job_config"]["filter"]["audio"]["encode_to_ims_ac4"]["drc"][
    #         "home_theatre_drc_profile"
    #     ] = payload.drc
    #     xml_base["job_config"]["filter"]["audio"]["encode_to_ims_ac4"]["drc"][
    #         "portable_hp_drc_profile"
    #     ] = payload.drc
    #     xml_base["job_config"]["filter"]["audio"]["encode_to_ims_ac4"]["drc"][
    #         "portable_spkr_drc_profile"
    #     ] = payload.drc
    # elif aformat == "thd":
    #     xml_base = xmltodict.parse(xml_thd_base)
    #     xml_base["job_config"]["output"]["mlp"]["storage"]["local"]["path"] = wpc(
    #         output, quote=True
    #     )
    #     xml_base["job_config"]["filter"]["audio"]["encode_to_dthd"][
    #         "atmos_presentation"
    #     ]["drc_profile"] = payload.drc
    #     xml_base["job_config"]["filter"]["audio"]["encode_to_dthd"][
    #         "presentation_8ch"
    #     ]["drc_profile"] = payload.drc
    #     xml_base["job_config"]["filter"]["audio"]["encode_to_dthd"][
    #         "presentation_6ch"
    #     ]["drc_profile"] = payload.drc
    #     xml_base["job_config"]["filter"]["audio"]["encode_to_dthd"][
    #         "presentation_2ch"
    #     ]["drc_profile"] = payload.drc
    #     xml_base["job_config"]["filter"]["audio"]["encode_to_dthd"][
    #         "custom_dialnorm"
    #     ] = payload.dialnorm
    # xml_base["job_config"]["input"]["audio"]["wav"]["storage"]["local"][
    #     "path"
    # ] = wpc(config["temp_path"], quote=True)
    # xml_base["job_config"]["misc"]["temp_dir"]["path"] = wpc(
    #     config["temp_path"], quote=True
    # )

    # # step 2 ?
    # #
    # #
    # xml = deepcopy(xml_base)
    # xml["job_config"]["input"]["audio"]["wav"]["file_name"] = basename(
    #     filelist[i], "wav", quote=True
    # )
    # if aformat == "ddp":
    #     xml["job_config"]["output"]["ec3"]["file_name"] = basename(
    #         filelist[i], "ec3", quote=True, stripdelay=True
    #     )
    #     if bitrate > 1024:
    #         xml["job_config"]["output"]["ec3"]["file_name"] = basename(
    #             filelist[i], "eb3", quote=True, stripdelay=True
    #         )
    #     if payload.force_standard:
    #         xml["job_config"]["output"]["ec3"]["file_name"] = basename(
    #             filelist[i], "ec3", quote=True, stripdelay=True
    #         )
    #     if payload.force_bluray:
    #         xml["job_config"]["output"]["ec3"]["file_name"] = basename(
    #             filelist[i], "eb3", quote=True, stripdelay=True
    #         )
    # elif aformat == "dd":
    #     xml["job_config"]["output"]["ec3"]["file_name"] = basename(
    #         filelist[i], "ac3", quote=True, stripdelay=True
    #     )
    #     xml["job_config"]["output"]["ac3"] = xml["job_config"]["output"]["ec3"]
    #     del xml["job_config"]["output"]["ec3"]
    # elif aformat == "ac4":
    #     xml["job_config"]["output"]["ac4"]["file_name"] = basename(
    #         filelist[i], "ac4", quote=True, stripdelay=True
    #     )
    # else:
    #     xml["job_config"]["output"]["mlp"]["file_name"] = basename(
    #         filelist[i], "thd", quote=True, stripdelay=True
    #     )

    # if aformat in ["dd", "ddp"]:
    #     delay_print, delay_xml, delay_mode = convert_delay_to_ms(
    #         delay, compensate=True
    #     )
    #     xml["job_config"]["filter"]["audio"]["pcm_to_ddp"][
    #         delay_mode
    #     ] = delay_xml
    # elif aformat in ["ac4"]:
    #     delay_print, delay_xml, delay_mode = convert_delay_to_ms(
    #         delay, compensate=True
    #     )
    #     xml["job_config"]["filter"]["audio"]["encode_to_ims_ac4"][
    #         delay_mode
    #     ] = delay_xml
    # else:
    #     delay_print, delay_xml, delay_mode = convert_delay_to_ms(
    #         delay, compensate=False
    #     )
    #     xml["job_config"]["filter"]["audio"]["encode_to_dthd"][
    #         delay_mode
    #     ] = delay_xml

    # save_xml(
    #     os.path.join(
    #         config["temp_path"], basename(filelist[i], "xml", sanitize=True)
    #     ),
    #     xml,
    # )
