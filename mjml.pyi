from typing import Literal, Protocol

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
    - include_loader: Fetch the included template using the path attribute. If it
        raises, `MjmlError` is raised with the original exception as `__cause__`.

    Warnings recorded while parsing are emitted as `MjmlWarning`. Use `parse()` to
    inspect them programmatically.

    Raises: MjmlError (a ValueError) if the template cannot be parsed or rendered.

    Returns: the rendered HTML string.
    """

MjmlErrorKind = Literal[
    "unexpected-element",
    "unexpected-token",
    "missing-attribute",
    "invalid-attribute",
    "invalid-format",
    "end-of-stream",
    "size-limit",
    "parser-error",
    "no-root-node",
    "include-loader-error",
    "unknown-fragment",
    "format",
    "inline-css",
]


class MjmlError(ValueError):
    """Raised when a template cannot be parsed or rendered.

    Subclasses `ValueError`. If an `include_loader` callback raises an `Exception`,
    the error has `kind == "include-loader-error"` and the callback's exception is
    its `__cause__`. Exceptions that are not `Exception`s (e.g. `KeyboardInterrupt`)
    propagate unchanged.
    """

    kind: MjmlErrorKind
    """What went wrong.

    Parsing (`origin` is the include path, or `None` for the root template;
    `start`/`end` are set where noted):
    - `"unexpected-element"`: an element that is not allowed here (`start`/`end` set).
    - `"unexpected-token"`: a token that is not allowed here (`start`/`end` set).
    - `"missing-attribute"`: a required attribute is missing (`start`/`end` set).
    - `"invalid-attribute"`: an attribute is malformed (`start`/`end` set).
    - `"invalid-format"`: the template text is malformed (`start`/`end` set).
    - `"end-of-stream"`: the template ended unexpectedly, e.g. an unclosed tag.
    - `"size-limit"`: the template is too large (the limit is 4 GiB).
    - `"parser-error"`: the underlying HTML parser rejected the template, e.g. an
      unquoted attribute value or an unterminated comment. For the root template
      `start == end` is the position where the parser gave up, and the message
      includes the parser's reason. For an included template MRML's position is
      relative to an internal wrapper and is not exposed, so `start`/`end` are
      `None` and the message has no position.
    - `"no-root-node"`: the template has no elements, e.g. an empty string.
      (`origin` is `None`.)
    - `"include-loader-error"`: the `include_loader` callback raised, returned
      something other than a `str`, or was not callable. `path` is the include
      path, `start`/`end` span the `mj-include` tag name in `origin`'s source, and
      `__cause__` is the original exception.

    Rendering (`origin`, `start` and `end` are `None`):
    - `"unknown-fragment"`: an internal lookup of a render fragment failed. This
      is an MRML bug, not a template mistake.
    - `"format"`: writing the HTML output failed.
    - `"inline-css"`: inlining `<mj-style inline="inline">` failed.
    """
    origin: str | None
    """Path of the included template the error came from, or `None` for the root
    template (or when MRML reports no origin)."""
    start: int | None
    """Start of the error's span as a character offset into the origin's source
    text (so `source[start:end]` works), or `None` when there is no span."""
    end: int | None
    """End of the error's span as a character offset into the origin's source
    text, or `None` when there is no span."""
    path: str | None
    """For `"include-loader-error"`: the `mj-include` path that failed to load.
    `None` for every other kind."""


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
    """Start of the warning's span as a character offset into the origin's source
    text (so `source[start:end]` works)."""
    end: int
    """End of the warning's span as a character offset into the origin's source
    text."""

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
    - include_loader: Fetch the included template using the path attribute. If it
        raises, `MjmlError` is raised with the original exception as `__cause__`.

    Returns: the parsed template, from which the title, preview and parse warnings
    can be read and which can be rendered to HTML.

    Raises: MjmlError (a ValueError) if the template cannot be parsed.
    """
