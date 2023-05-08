import sys

exit_success = 0
exit_fail = 1


def _exit_application(msg: str, exit_code: int = 0):
    """A clean way to exit the program without raising traceback errors

    Args:
        msg (str): Success or Error message you'd like to display in the console
        exit_code (int): Can either be 0 (success) or 1 (fail)
    """
    if exit_code not in {0, 1}:
        raise ValueError("exit_code must only be '0' or '1' (int)")

    if exit_code == 0:
        output = sys.stdout
    elif exit_code == 1:
        output = sys.stderr

    print(msg, file=output)
    sys.exit(exit_code)
