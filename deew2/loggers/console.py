class PrintSameLine:
    """Class to correctly print on same line"""

    def __init__(self):
        self.last_message = ""

    def print_msg(self, msg: str):
        print(" " * len(self.last_message), end="\r", flush=True)
        print(msg, end="\r", flush=True)
        self.last_message = msg
