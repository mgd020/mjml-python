from typing import Protocol

class IncludeLoader(Protocol):
    def __call__(self, path: str) -> str: ...

def mjml2html(
    input: str,
    *,
    disable_comments: bool = False,
    social_icon_origin: str | None = None,
    fonts: dict[str, str] | None = None,
    include_loader: IncludeLoader | None = None,
) -> str:
    """Convert MJML string to HTML string.

    Params:
    - input: The input MJML string
    - disable_comments: Strip comments out of rendered HTML
    - social_icon_origin: Custom URL origin for social icons. Icon name is appended
        (e.g. `facebook.png`) as-is, so include a trailing slash.
    - fonts: Fonts imported in the HTML rendered by MJML.
    - include_loader: Fetch the included template using the path attribute.

    Warnings recorded while parsing are emitted as `MjmlWarning`. Use `parse()` to
    inspect them programmatically.

    Returns: the rendered HTML string.
    """

class MjmlWarning(UserWarning):
    """A warning MRML recorded while parsing a template, e.g. a misspelt attribute.

    `mjml2html()` emits these via the `warnings` module (use
    `warnings.simplefilter("error", MjmlWarning)` to turn them into errors);
    `parse()` returns them in `Mjml.warnings`.
    """

    kind: str
    """Warning kind, e.g. `"unexpected-attribute"` or `"inline-style-unsupported"`."""
    origin: str | None
    """Path of the included template the warning came from, or `None` for the root template."""
    start: int
    """Start offset of the warning's span in the origin's source text."""
    end: int
    """End offset of the warning's span in the origin's source text."""

class Mjml:
    """A parsed MJML template. Create one with `parse()`."""

    @property
    def title(self) -> str | None:
        """The contents of `<mj-title>`, if any."""

    @property
    def preview(self) -> str | None:
        """The contents of `<mj-preview>`, if any."""

    @property
    def warnings(self) -> list[MjmlWarning]:
        """Warnings recorded while parsing."""

    def render(
        self,
        *,
        disable_comments: bool = False,
        social_icon_origin: str | None = None,
        fonts: dict[str, str] | None = None,
    ) -> str:
        """Render the template to an HTML string.

        Params:
        - disable_comments: Strip comments out of rendered HTML
        - social_icon_origin: Custom URL origin for social icons. Icon name is appended
            (e.g. `facebook.png`) as-is, so include a trailing slash.
        - fonts: Fonts imported in the HTML rendered by MJML.
        """

def parse(
    input: str,
    *,
    include_loader: IncludeLoader | None = None,
) -> Mjml:
    """Parse an MJML string.

    Params:
    - input: The input MJML string
    - include_loader: Fetch the included template using the path attribute.

    Returns: the parsed template, from which the title, preview and parse warnings
    can be read and which can be rendered to HTML.

    Raises: ValueError if the template cannot be parsed.
    """
