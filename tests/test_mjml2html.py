import sys
import threading
import time
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
        self.assertIn("unexpected token", str(ctx.exception))


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
        self.assertNotIn("https://example.com/fonts?family=Honk", result)
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

    def test_head_override(self):
        result = mjml2html(
            self.string.replace(
                "<mjml>",
                """
                    <mjml><mj-head>
                    <mj-font name="Honk" href="https://example.com/fonts?family=Honk"></mj-font>
                    <mj-font name="Open Sans" href="https://example.com/fonts?family=Open+Sans"></mj-font>
                    </mj-head>
                """,
            )
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
        self.assertIn("unable to load included template", str(ctx.exception))

    def test_head_attributes(self):
        strings = {"inner.mjml": self.inner_string, "text.mjml": self.text_string}
        result = mjml2html(self.outer_string, include_loader=strings.__getitem__)
        expected = mjml2html(
            self.outer_string.replace(self.include_string, self.inner_string),
            include_loader=strings.__getitem__,
        )
        self.assertEqual(result, expected)

    def test_include_css(self):
        mjml = """
        <mjml>
            <mj-head>
                <mj-include path="styles.css" type="css" />
            </mj-head>
        </mjml>
        """
        strings = {"styles.css": ".some-class { font-family: monospace }"}
        result = mjml2html(mjml, include_loader=strings.__getitem__)
        self.assertRegex(
            result,
            r"<style type=\"text/css\">.some-class { font-family: monospace }</style>",
        )

    def test_include_at_root(self):
        strings = {
            "body.mjml": """
                <mj-body>
                    <mj-section>
                        <mj-column>
                            <mj-text>Hello from the root include</mj-text>
                        </mj-column>
                    </mj-section>
                </mj-body>
            """
        }
        result = mjml2html(
            """
            <mjml>
                <mj-include path="body.mjml" />
            </mjml>
            """,
            include_loader=strings.__getitem__,
        )
        self.assertRegex(result, r"^<!doctype html>")
        self.assertRegex(result, r">Hello from the root include<")


class TestReleasesGil(unittest.TestCase):
    def test_other_threads_run_during_compile(self):
        section = (
            "<mj-section><mj-column><mj-text>Row</mj-text></mj-column></mj-section>"
        )
        doc = "<mjml><mj-body>" + section * 2_000 + "</mj-body></mjml>"

        ticks = 0
        stop = False

        def spin():
            nonlocal ticks
            while not stop:
                ticks += 1
                time.sleep(0)  # hands the GIL back on every iteration

        thread = threading.Thread(target=spin, daemon=True)
        old_interval = sys.getswitchinterval()
        # With no forced switches, the spinner can only run during the call
        # if mjml2html itself releases the GIL.
        sys.setswitchinterval(100)
        try:
            thread.start()
            while ticks == 0:
                time.sleep(0.001)
            before = ticks
            mjml2html(doc)
            ran = ticks - before
        finally:
            stop = True
            thread.join()
            sys.setswitchinterval(old_interval)

        self.assertGreater(ran, 0, "mjml2html did not release the GIL")


class TestInlineStyle(unittest.TestCase):
    def test_mj_style_inline_is_inlined(self):
        result = mjml2html(
            """
            <mjml>
              <mj-head>
                <mj-style inline="inline">.alert { color: red; }</mj-style>
              </mj-head>
              <mj-body>
                <mj-section><mj-column>
                  <mj-text><span class="alert">Payment failed</span></mj-text>
                </mj-column></mj-section>
              </mj-body>
            </mjml>
            """
        )
        self.assertIn('<span class="alert" style="color: red;">', result)
