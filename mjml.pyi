from typing import Optional, Dict


def mjml2html(
    input: str,
    *,
    disable_comments: bool = False,
    social_icon_origin: Optional[str] = None,
    fonts: Optional[Dict[str, str]] = None,
) -> str:
    """Convert MJML string to HTML string."""
