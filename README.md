# `mjml-python`

Compile MJML at runtime **without** a Node.js service, external API, or subprocess.  
`mjml-python` is a Python wrapper around the Rust MJML engine [MRML](https://github.com/jdrouet/mrml), a high-performance port of the official [MJML](https://github.com/mjmlio/mjml).

## Why

Using MJML traditionally requires either:

- running the Node.js-based MJML CLI as a subprocess, or
- calling an external MJML API service.

Both approaches add memory overhead, latency, operational complexity, and cost.

From [MRML](https://github.com/jolimail/mrml#why):

> A Node.js server rendering an MJML template takes around 20 MB of RAM at startup and 130 MB under stress test. In Rust, less than 1.7 MB at startup and a bit less that 3 MB under stress test. The Rust version can also handle twice as many requests per second.

By embedding the Rust engine directly, `mjml-python` avoids all external dependencies while staying compatible with most of MJML (see [Known limitations](#known-limitations)).

## Installation

Install from [PyPI](https://pypi.org/project/mjml-python/):

```sh
pip install mjml-python
```

## Usage

Call `mjml2html()` with your MJML string:

```py
from mjml import mjml2html

html = mjml2html(
    '''
        <mjml>
          <mj-body>
            <mj-section>
              <mj-column>
                <mj-image width="100px" src="/assets/img/logo-small.png"></mj-image>
                <mj-divider border-color="#F45E43"></mj-divider>
                <!-- Say hello to the user -->
                <mj-text font-size="20px" color="#F45E43" font-family="Open Sans">Hello World</mj-text>
              </mj-column>
            </mj-section>
            <mj-section>
              <mj-column>
                <mj-social font-size="15px" icon-size="30px" mode="horizontal">
                  <mj-social-element name="facebook" href="https://mjml.io/">
                    Facebook
                  </mj-social-element>
                  <mj-social-element name="google" href="https://mjml.io/">
                    Google
                  </mj-social-element>
                  <mj-social-element  name="twitter" href="https://mjml.io/">
                    Twitter
                  </mj-social-element>
                </mj-social>
              </mj-column>
            </mj-section>
          </mj-body>
        </mjml>
    ''',
    disable_comments=True,
    social_icon_origin="https://example.com/",
    fonts={
        "Open Sans": "https://fonts.googleapis.com/css?family=Open+Sans:300,400,500,700",
        "Ubuntu": "https://fonts.googleapis.com/css?family=Ubuntu:300,400,500,700",
    }
)
```

**Warnings**

`mjml2html()` reports problems MRML finds while parsing (for example a misspelt attribute) through Python's `warnings` module as `mjml.MjmlWarning`, a `UserWarning` subclass. To fail on them:

```py
import warnings
from mjml import MjmlWarning

warnings.simplefilter("error", MjmlWarning)
```

**Inspecting a template before rendering**

`parse()` returns a parsed template exposing the `<mj-title>`, `<mj-preview>` and any warnings MRML recorded while parsing (for example a misspelt attribute). Call `render()` on it to get the HTML; `render()` takes the same `disable_comments`, `social_icon_origin` and `fonts` options as `mjml2html()`.

```py
from mjml import parse

template = parse('<mjml lnag="fr"><mj-head><mj-title>Shipped</mj-title></mj-head><mj-body></mj-body></mjml>')
template.title     # "Shipped"
template.preview   # None
template.warnings  # [MjmlWarning("unexpected-attribute at 6:15 in root template")]
html = template.render(disable_comments=True)
```

**Example using Django templates**

```py
from django.core.mail import send_mail
from django.template.loader import render_to_string
from mjml import mjml2html

context = {'foo': 'bar'}
text_message = render_to_string('my_text_template.txt', context)
html_message = mjml2html(render_to_string('my_mjml_template.mjml', context))
send_mail(
    'Subject here',
    text_message,
    'from@example.com',
    ['to@example.com'],
    fail_silently=False,
    html_message=html_message,
)
```

## Configuration Options

`mjml-python` supports the following options:

| Name                 | Type                           | Default value | Comment                                                                          |
|----------------------|--------------------------------|---------------|----------------------------------------------------------------------------------|
| `disable_comments`   | `bool`                         | `False`       | Strip comments out of rendered HTML                                              |
| `social_icon_origin` | `str \| None`                  | `None`        | Custom URL origin for social icons. Icon name is appended as-is (e.g. `facebook.png`), so include a trailing slash. |
| `fonts`              | `dict[str, str] \| None`       | `None`        | Fonts imported in the HTML rendered by MJML.                                     |
| `include_loader`     | `Callable[[str], str] \| None` | `None`        | Fetch the included template using the path attribute.                            |

**Default fonts** (used when `fonts=None`):

```py
{
    "Open Sans": "https://fonts.googleapis.com/css?family=Open+Sans:300,400,500,700",
    "Droid Sans": "https://fonts.googleapis.com/css?family=Droid+Sans:300,400,500,700",
    "Lato": "https://fonts.googleapis.com/css?family=Lato:300,400,500,700",
    "Roboto": "https://fonts.googleapis.com/css?family=Roboto:300,400,500,700",
    "Ubuntu": "https://fonts.googleapis.com/css?family=Ubuntu:300,400,500,700",
}       
```

## Known limitations

`mjml-python` renders with [MRML](https://github.com/jdrouet/mrml), which covers most of MJML but not all of it. Differences to be aware of:

- `mj-html-attributes` and `mj-selector` are not supported; a template using them fails to compile with a `ValueError`.
- `<mj-raw position="file-start">` placed directly under `<mjml>` fails to parse.
- There is no equivalent of MJML's `validationLevel`. Unknown tags such as `<mj-txet>` are copied into the output, and unknown attributes are dropped. MRML only reports a limited set of mistakes (e.g. an unexpected attribute on `<mjml>`), and `mjml-python` surfaces those as `MjmlWarning`.
- There are no `minify` or `beautify` options.
- MRML 6.0.1 is tested against MJML 4.18 output. Alignment with MJML 5 is not released yet.

## Why choose `mjml-python` instead of other MJML packages?

- **Simple, Pythonic API** – a single `mjml2html()` function that takes plain keyword arguments and returns a clean HTML string. No need to construct parser/render option objects or work with custom result classes.

- **Batteries-included defaults** – sensible defaults for fonts, comment handling, and include behavior, so most templates work with zero configuration.

- **Flexible include loader** – pass a Python callable to handle `<mj-include>` however you like (filesystem, database, HTTP, etc.) without learning MRML’s multiple loader types.

- **Minimal surface area** – intentionally focused on the common case: take MJML input → return production-ready HTML. No extra abstractions.

- **No external services or subprocesses required** – rendering is performed entirely inside the Python extension using the embedded Rust MJML engine. No Node.js installation, MJML CLI, or HTTP API needed.

- **Stable wheel builds** – lightweight ABI-3 wheels for all major platforms (CPython 3.8+), fast installation, and no compilation required on typical environments.

**In short:** `mjml-python` is ideal when you want a straightforward, Python-friendly way to render MJML without dealing with the full MRML configuration surface or external tooling.

## Development

With [Nix](https://nixos.org/): 

```sh
nix-shell
```

With Python 3.8+, uv, Rust 1.83+ and Cargo installed:

```sh
uv sync --locked --no-install-project
uv run --locked --no-sync maturin develop
uv run --locked --no-sync python -m unittest
```
