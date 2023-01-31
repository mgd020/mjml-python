# mjml-python

A Python wrapper for [MRML](https://github.com/jolimail/mrml-core) (Rust port of [MJML](https://github.com/mjmlio/mjml)).

## Installation

```sh
pip install mjml-python
```

## Usage

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

## Development

```sh
python -m venv env
. env/bin/activate
pip install -r requirements.txt
maturin develop
```
