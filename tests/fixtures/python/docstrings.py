"""Separates docstrings from f-string and bytes statements."""


def label(x):
    "Item " f"{x}"
    return x


def formatted(x):
    f"Item {x}"
    return x


def raw_bytes():
    b"raw"
    return 1


def plain():
    r"Raw docstring."
    ​# a zero-width space before a comment
    return 1
