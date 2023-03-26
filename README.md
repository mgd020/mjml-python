# mjml-python

Compile MJML at runtime without an external Node service/process. It is a Python wrapper for [MRML](https://github.com/jolimail/mrml-core) (Rust port of [MJML](https://github.com/mjmlio/mjml)).

## Why

From [MRML](https://github.com/jolimail/mrml-core#why):

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

html = mjml2html('''
<mjml>
  <mj-body>
    <mj-section>
      <mj-column>
        <mj-image width="100px" src="/assets/img/logo-small.png"></mj-image>
        <mj-divider border-color="#F45E43"></mj-divider>
        <mj-text font-size="20px" color="#F45E43" font-family="helvetica">Hello World</mj-text>
      </mj-column>
    </mj-section>
  </mj-body>
</mjml>
''')
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

## Development

```sh
python -m venv env
. env/bin/activate
pip install -r requirements.txt
maturin develop
```
