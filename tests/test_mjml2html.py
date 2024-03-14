import unittest
from mjml import mjml2html


class TestSimple(unittest.TestCase):
    def test_good(self):
        result = mjml2html(
            """
                <mjml><mj-body><mj-section><mj-column>
                <mj-text>Hello World</mj-text>
                </mj-column></mj-section></mj-body></mjml>
            """
        )
        self.assertRegex(result, r"^<!doctype html>")
        self.assertRegex(result, r">Hello World<")

    def test_bad(self):
        with self.assertRaises(ValueError) as ctx:
            mjml2html("""<mj-text>Hello World</mj-text>""")
        self.assertEqual(str(ctx.exception), "unexpected token at position 9..20")


class TestDisableComments(unittest.TestCase):
    string = """
        <mjml><mj-body><mj-section><mj-column>
        <!-- this is a comment -->
        <mj-text>Hello World</mj-text>
        </mj-column></mj-section></mj-body></mjml>
    """

    def test_false(self):
        result = mjml2html(self.string, disable_comments=False)
        self.assertIn("<!-- this is a comment -->", result)

    def test_true(self):
        result = mjml2html(self.string, disable_comments=True)
        self.assertNotIn("<!-- this is a comment -->", result)


class TestSocialIconOrigin(unittest.TestCase):
    string = """
        <mjml><mj-body><mj-section><mj-column>
        <mj-social><mj-social-element name="facebook" href="https://example.invalid/"/></mj-social>
        </mj-column></mj-section></mj-body></mjml>
    """

    def test_default(self):
        result = mjml2html(self.string, social_icon_origin=None)
        self.assertIn(
            'href="https://www.facebook.com/sharer/sharer.php?u=https://example.invalid/"',
            result,
        )
        self.assertIn(
            'src="https://www.mailjet.com/images/theme/v1/icons/ico-social/facebook.png"',
            result,
        )

    def test_override(self):
        result = mjml2html(self.string, social_icon_origin="https://example.invalid/")
        self.assertIn(
            'href="https://www.facebook.com/sharer/sharer.php?u=https://example.invalid/"',
            result,
        )
        self.assertIn('src="https://example.invalid/facebook.png"', result)


class TestFonts(unittest.TestCase):
    string = """
        <mjml><mj-body><mj-section><mj-column>
        <mj-text font-family="Honk">Honk</mj-text>
        <mj-text font-family="Open Sans">Open Sans</mj-text>
        <mj-text font-family="Missing">Missing</mj-text>
        </mj-column></mj-section></mj-body></mjml>
    """

    def test_default(self):
        result = mjml2html(self.string, fonts=None)
        self.assertIn("https://fonts.googleapis.com/css?family=Open+Sans", result)

    def test_override(self):
        result = mjml2html(
            self.string,
            fonts={
                "Honk": "https://example.com/fonts?family=Honk",
                "Open Sans": "https://example.com/fonts?family=Open+Sans",
            },
        )
        self.assertIn("https://example.com/fonts?family=Honk", result)
        self.assertIn("https://example.com/fonts?family=Open+Sans", result)


class TestIncludeLoader(unittest.TestCase):
    include_string = '<mj-include path="inner.mjml" />'

    outer_string = f"""
        <mjml>
            <mj-head>
                {include_string}
            </mj-head>
            <mj-body>
                <mj-section>
                <mj-column>
                    <mj-include path="text.mjml" />
                </mj-column>
                </mj-section>
            </mj-body>
        </mjml>
    """

    inner_string = """
        <mj-attributes>
            <mj-text padding="0" />
            <mj-class name="blue" color="blue" />
            <mj-class name="big" font-size="20px" />
            <mj-all font-family="Arial" />
        </mj-attributes>
    """

    text_string = """
        <mj-text mj-class="blue big">Hello World</mj-text>
    """

    def test_ok(self):
        strings = {"inner.mjml": self.inner_string, "text.mjml": self.text_string}
        result = mjml2html(self.outer_string, include_loader=strings.__getitem__)
        self.assertRegex(result, r"^<!doctype html>")
        self.assertRegex(result, r">Hello World<")

    def test_missing(self):
        strings = {}
        with self.assertRaises(ValueError) as ctx:
            mjml2html(self.outer_string, include_loader=strings.__getitem__)
        self.assertEqual(str(ctx.exception), "unable to load included template")

    def test_head_attributes(self):
        strings = {"inner.mjml": self.inner_string, "text.mjml": self.text_string}
        result = mjml2html(self.outer_string, include_loader=strings.__getitem__)
        expected = mjml2html(
            self.outer_string.replace(self.include_string, self.inner_string),
            include_loader=strings.__getitem__,
        )
        self.assertEqual(result, expected)
