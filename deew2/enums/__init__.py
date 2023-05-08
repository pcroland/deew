from argparse import ArgumentTypeError
from enum import Enum


def case_insensitive_enum(enum_class):
    """Return a converter that takes a string and returns the corresponding
    enumeration value, regardless of case.

    Args:
        enum_class (Enum): The enumeration class to convert values to.

    Returns:
        A converter function that takes a string and returns the corresponding
        enumeration value.

    Raises:
        ArgumentTypeError: If the input string is not a valid choice
        for the given enumeration class.
    """

    def converter(value):
        try:
            if value.isdigit():
                return enum_class(int(value))
            else:
                return enum_class[value.upper()]
        except (KeyError, ValueError):
            raise ArgumentTypeError(f"Invalid choice: {value}")

    return converter


def enum_choices(enum_class: Enum) -> str:
    """
    Returns a string representation of all possible choices in the given enumeration class.

    Args:
        enum_class (Enum): The enumeration class to retrieve choices from.

    Returns:
        A string with the format "{choice1[choice_value1]},{choice2[choice_value2]},...".

    Example:
        If the enumeration class is defined as follows:

        class DolbyDigitalChannels(Enum):
            MONO = 1
            STEREO = 2
            SURROUND = 6

        The function will return the following string:

        "{MONO[1]},{STEREO[2]},{SURROUND[6]}"
    """
    return f"{{{','.join(e.name+'['+str(e.value)+']' for e in enum_class)}}}"
