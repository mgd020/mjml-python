use mrml;
use mrml::prelude::parser::loader::*;
use mrml::prelude::parser::*;
use mrml::prelude::render::*;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use std::borrow::Cow;
use std::collections::HashMap;
use std::io::ErrorKind;

#[derive(Debug)]
struct CallbackIncludeLoader(pub PyObject);

impl IncludeLoader for CallbackIncludeLoader {
    fn resolve(&self, path: &str) -> Result<String, IncludeLoaderError> {
        Python::with_gil(|py| {
            self.0
                .call1(py, (path,))
                .and_then(|o| o.extract::<String>(py))
                .map_err(|_| IncludeLoaderError::new(path, ErrorKind::Other))
        })
    }
}

#[pyfunction]
#[pyo3(signature = (input, *, disable_comments=false, social_icon_origin=None, fonts=None, include_loader=None))]
fn mjml2html(
    py: pyo3::Python<'_>,
    input: String,
    disable_comments: bool,
    social_icon_origin: Option<String>,
    fonts: Option<HashMap<String, String>>,
    include_loader: Option<PyObject>,
) -> PyResult<String> {
    py.allow_threads(|| {
        let parse_opts = ParserOptions {
            include_loader: match include_loader {
                None => Box::new(noop_loader::NoopIncludeLoader),
                Some(item) => Box::new(CallbackIncludeLoader(item)),
            },
        };

        let root = mrml::parse_with_options(input, &parse_opts)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;

        let fonts_map = match fonts {
            None => RenderOptions::default().fonts,
            Some(item) => item
                .into_iter()
                .map(|(k, v)| (k, Cow::Owned(v)))
                .collect(),
        };

        let render_opts = RenderOptions {
            disable_comments,
            social_icon_origin: social_icon_origin.map(Into::into),
            fonts: fonts_map,
        };

        root.element
            .render(&render_opts)
            .map_err(|e| PyValueError::new_err(e.to_string()))
    })
}

#[pymodule]
fn mjml(_py: Python<'_>, m: &Bound<PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(mjml2html, m)?)?;
    Ok(())
}
