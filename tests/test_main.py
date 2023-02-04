import pathlib

import pytest
import mjml


@pytest.mark.parametrize(
    'template_path, expected_path',
    [
        ('tests/files/case1_template.mjml', 'tests/files/case1_result.html'),
    ]
)
def test_mjml2html(template_path, expected_path):
    """Basic test for mjml2html function."""
    template = pathlib.Path(template_path).read_text()
    expected = pathlib.Path(expected_path).read_text()

    html = mjml.mjml2html(template)

    assert html == expected


@pytest.mark.parametrize(
    'template_path, expected_path',
    [
        ('tests/files/case1_template.mjml', 'tests/files/case1_result.html'),
    ]
)
def test_parse_render(template_path, expected_path):
    """Test for parse and render function."""
    template = pathlib.Path(template_path).read_text()
    expected = pathlib.Path(expected_path).read_text()

    m = mjml.parse(template)    
    html = m.render()
    
    assert html == expected


def test_parse_error():
    """Test for parse error."""
    with pytest.raises(ValueError):
        mjml.parse('</mjml>')

def test_mjml_title_property():
    """Test for get_title function."""
    m = mjml.parse('''
      <mjml>
        <mj-head>
          <mj-title>Hello MJML Title</mj-title>
        </mj-head>
      </mjml>
    ''')
    assert m.title == 'Hello MJML Title'


def test_mjml_preview_property():
    """Test for get_title function."""
    m = mjml.parse('''
      <mjml>
       <mj-head>
         <mj-preview>Hello MJML Preview</mj-preview>
       </mj-head>
      </mjml>
    ''')
    assert m.preview == 'Hello MJML Preview'


def test_mjml_template_property():
    """Test for get_title function."""
    template = pathlib.Path('tests/files/case1_template.mjml').read_text()
    m = mjml.parse(template)
    assert m.template == template
