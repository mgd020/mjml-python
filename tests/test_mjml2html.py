import sys
import threading
import time
import unittest
import warnings

from mjml import MjmlWarning, mjml2html, parse


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


class TestParse(unittest.TestCase):
    string = """
        <mjml lnag="fr">
          <mj-head>
            <mj-title>Your order has shipped</mj-title>
            <mj-preview>Arriving on Thursday</mj-preview>
          </mj-head>
          <mj-body>
            <mj-section><mj-column>
              <!-- a comment -->
              <mj-text>Bonjour</mj-text>
            </mj-column></mj-section>
          </mj-body>
        </mjml>
    """

    def test_title_and_preview(self):
        parsed = parse(self.string)
        self.assertEqual(parsed.title, "Your order has shipped")
        self.assertEqual(parsed.preview, "Arriving on Thursday")

    def test_no_title_or_preview(self):
        parsed = parse("<mjml><mj-body></mj-body></mjml>")
        self.assertIsNone(parsed.title)
        self.assertIsNone(parsed.preview)
        self.assertEqual(parsed.warnings, [])

    def test_warnings(self):
        (warning,) = parse(self.string).warnings
        self.assertIsInstance(warning, MjmlWarning)
        self.assertEqual(warning.kind, "unexpected-attribute")
        self.assertIsNone(warning.origin)
        self.assertEqual(self.string[warning.start : warning.end], 'lnag="fr"')

    def test_warning_origin_is_include_path(self):
        strings = {"head.mjml": '<mj-head><mj-style foo="bar">a{}</mj-style></mj-head>'}
        parsed = parse(
            '<mjml><mj-include path="head.mjml" /><mj-body></mj-body></mjml>',
            include_loader=strings.__getitem__,
        )
        self.assertEqual([w.origin for w in parsed.warnings], ["head.mjml"])

    def test_render_matches_mjml2html(self):
        with warnings.catch_warnings():
            warnings.simplefilter("ignore", MjmlWarning)
            expected = mjml2html(self.string)
        self.assertEqual(parse(self.string).render(), expected)

    def test_render_options(self):
        parsed = parse(self.string)
        self.assertIn("a comment", parsed.render())
        self.assertNotIn("a comment", parsed.render(disable_comments=True))
        with warnings.catch_warnings():
            warnings.simplefilter("ignore", MjmlWarning)
            expected = mjml2html(self.string, disable_comments=True)
        self.assertEqual(parsed.render(disable_comments=True), expected)

    def test_render_many_times(self):
        parsed = parse(self.string)
        self.assertEqual(parsed.render(), parsed.render())

    def test_bad(self):
        with self.assertRaises(ValueError):
            parse("<mj-text>Hello World</mj-text>")


class TestWarnings(unittest.TestCase):
    bad = '<mjml lnag="fr"><mj-body></mj-body></mjml>'

    def test_mjml2html_warns(self):
        with self.assertWarnsRegex(MjmlWarning, r"unexpected-attribute at 6:15 in root"):
            result = mjml2html(self.bad)
        self.assertRegex(result, r"^<!doctype html>")

    def test_emitted_warning_has_attributes(self):
        with warnings.catch_warnings(record=True) as caught:
            warnings.simplefilter("always")
            mjml2html(self.bad)
        (w,) = caught
        self.assertIs(w.category, MjmlWarning)
        self.assertEqual(w.message.kind, "unexpected-attribute")
        self.assertEqual(self.bad[w.message.start : w.message.end], 'lnag="fr"')

    def test_warning_is_user_warning(self):
        self.assertTrue(issubclass(MjmlWarning, UserWarning))

    def test_warning_can_be_error(self):
        with warnings.catch_warnings():
            warnings.simplefilter("error", MjmlWarning)
            with self.assertRaises(MjmlWarning):
                mjml2html(self.bad)

    def test_no_warning_when_clean(self):
        with warnings.catch_warnings():
            warnings.simplefilter("error")
            mjml2html("<mjml><mj-body></mj-body></mjml>")

    def test_parse_does_not_warn(self):
        with warnings.catch_warnings():
            warnings.simplefilter("error")
            parse(self.bad)
