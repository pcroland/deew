import xmltodict
from typing import Any


def save_xml(f: str, xml: dict[str, Any]) -> None:
    with open(f, "w", encoding="utf-8") as fd:
        fd.write(xmltodict.unparse(xml, pretty=True, indent="  "))
