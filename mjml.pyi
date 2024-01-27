from typing import Optional, Dict, Protocol


class IncludeLoader(Protocol):
    def __call__(self, path: str) -> str: ...


def mjml2html(
    input: str,
    *,
    disable_comments: bool = False,
    social_icon_origin: Optional[str] = None,
    fonts: Optional[Dict[str, str]] = None,
    include_loader: Optional[IncludeLoader] = None,
) -> str:
    """Convert MJML string to HTML string.

    Params:
    - input: The input MJML string
    - disable_comments: Strip comments out of rendered HTML
    - social_icon_origin: Custom URL origin for social icons. Icon name is appended
        (e.g. `facebook.png`).
    - fonts: Fonts imported in the HTML rendered by MJML.
    - include_loader: Fetch the included template using the path attribute.

    Returns: the rendered HTML string.
    """
