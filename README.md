# mjml-python

Compile MJML at runtime without an external Node service/process. It is a Python wrapper for [MRML](https://github.com/jdrouet/mrml) (Rust port of [MJML](https://github.com/mjmlio/mjml)).

## Why

From [MRML](https://github.com/jolimail/mrml#why):

> A Node.js server rendering an MJML template takes around 20 MB of RAM at startup and 130 MB under stress test. In Rust, less than 1.7 MB at startup and a bit less that 3 MB under stress test. The Rust version can also handle twice as many requests per second.

All of that is without considering http transaction cost when using a  node service or process.

## How

Install from [PyPI](https://pypi.org/project/mjml-python/):

```sh
pip install mjml-python
```

Import `mjml2html` and pass a string to compile: 

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
    social_icon_origin="https://example.com",
    fonts={
        "Open Sans": "https://fonts.googleapis.com/css?family=Open+Sans:300,400,500,700",
        "Ubuntu": "https://fonts.googleapis.com/css?family=Ubuntu:300,400,500,700",
    })
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

**Options**

`mjml-python` supports the following options:

| Name                 | Type                     | Default value | Comment                                                                          |
|----------------------|--------------------------|---------------|----------------------------------------------------------------------------------|
| `disable_comments`   | `bool`                   | `False`       | Strip comments out of rendered HTML                                              |
| `social_icon_origin` | `str \| None`            | `None`        | Custom URL origin for social icons. Icon name is appended (e.g. `facebook.png`). |
| `fonts`              | `dict[str, str] \| None` | `None`        | Fonts imported in the HTML rendered by MJML.                                     |

*Notes* :

- When `fonts` option is set to `None`, the following default fonts will be used: 
  ```py
  {
      "Open Sans": "https://fonts.googleapis.com/css?family=Open+Sans:300,400,500,700",
      "Droid Sans": "https://fonts.googleapis.com/css?family=Droid+Sans:300,400,500,700",
      "Lato": "https://fonts.googleapis.com/css?family=Lato:300,400,500,700",
      "Roboto": "https://fonts.googleapis.com/css?family=Roboto:300,400,500,700",
      "Ubuntu": "https://fonts.googleapis.com/css?family=Ubuntu:300,400,500,700",
  }       
  ```

## Development

```sh
python -m venv env
. env/bin/activate
pip install -r requirements.txt
maturin develop
```
