import re
from datetime import timedelta
from deeaw2.exceptions import InvalidDelayError
from deeaw2.enums.shared import DeeDelay, DeeDelayModes


class DelayGenerator:
    def get_dee_delay(self, delay: str, compensate: bool = True):
        """
        Converts the delay string to the proper format, checks for invalid characters,
        and returns a tuple containing the Dee Delay mode and the delay in the
        appropriate format for Dee Delay.

        Parameters:
            delay (str): The delay string to be converted.

        Returns:
            DeeDelay dataclass with needed values for XML.

        Raises:
            InvalidDelayError: If the delay input contains invalid characters or is
            not in the correct format.

        Example:
            ```
            dee = DeeDelay()
            dee.get_dee_delay('-2s')
            ```
        """
        # check for invalid characters in string
        self._check_for_invalid_characters(delay)

        # convert delay to proper format
        get_delay = self._convert_delay_ms(delay)

        # get only numbers from delay
        s_delay = re.search(r"\d+\.?\d*", get_delay)

        # Define the number of samples in Dolby's silence offset
        DOLBY_SILENCE_OFFSET = 256

        # if numbers was detected
        if s_delay:
            # convert numbers to a float
            s_delay = float(s_delay.group())

            # Subtract the Dolby silence offset
            s_delay -= (DOLBY_SILENCE_OFFSET / 48000) * 1000
            # s_delay -= 16 / 3

            # if delay is set to "-"
            if s_delay < 0:
                dee_delay_mode = DeeDelayModes.NEGATIVE
                delay_xml = str(timedelta(seconds=(abs(s_delay) / 1000)))
                if "." not in delay_xml:
                    delay_xml = f"{delay_xml}.0"

            # if delay is positive
            elif s_delay > 0:
                dee_delay_mode = DeeDelayModes.POSITIVE
                delay_xml = format(s_delay / 1000, ".6f")

            # create an internal data class
            data_class = DeeDelay(dee_delay_mode, delay_xml)

            return data_class


        # if no numbers was detected raise an error
        elif not s_delay:
            raise InvalidDelayError(
                "Delay input must be in the format of -10ms/10ms or -10s/10s"
            )

    @staticmethod
    def _convert_delay_ms(delay: str):
        """
        Converts the delay string to milliseconds.

        Args:
            delay (str): A delay string in the format of -10ms/10ms or -10s/10s.

        Returns:
            str: The delay string in milliseconds.
        """
        # lower delay string
        lowered_input = delay.lower()

        # set negative string
        negative = ""
        if "-" in lowered_input:
            negative = "-"

        # check if input is in milliseconds
        if "ms" in lowered_input:
            ms_delay = re.search(r"\d+", lowered_input)
            if ms_delay:
                ms_delay = float(ms_delay.group())
            else:
                raise InvalidDelayError(
                    "Delay input must be in the format of -10ms/10ms or -10s/10s"
                )
            return f"{negative}{ms_delay}ms"

        # check if input is in seconds
        elif "s" in lowered_input:
            s_delay = re.search(r"\d+\.?\d*", lowered_input)
            if s_delay:
                s_delay = float(s_delay.group())
            else:
                raise InvalidDelayError(
                    "Delay input must be in the format of -10ms/10ms or -10s/10s"
                )
            seconds_to_milliseconds = s_delay * 1000
            return f"{negative}{seconds_to_milliseconds}ms"

    @staticmethod
    def _check_for_invalid_characters(delay: str):
        """
        Check if a delay string contains any invalid characters.

        The delay string must be in the format of '-10ms/10ms' or '-10s/10s', where
        the leading '-' is optional, the number is an integer or a decimal with up
        to three digits, and the unit is 'ms' for milliseconds or 's' for seconds.

        If the delay string contains any characters other than digits, '-', 'm',
        's', or whitespace, an InvalidDelayError is raised with a message that
        includes the invalid characters.

        Parameters:
            delay (str): The delay string to check.

        Raises:
            InvalidDelayError: If the delay string contains any invalid characters.
        """
        invalid_chars = re.findall(r"[^\-\sms\d]", delay.lower())
        if invalid_chars:
            raise InvalidDelayError(
                f"Invalid characters detected: {', '.join(invalid_chars)}\n"
                "Delay input must be in the format of -10ms/10ms or -10s/10s"
            )
        if re.search(r"\s", delay):
            raise InvalidDelayError("Delay input cannot contain whitespace characters.")
