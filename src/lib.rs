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
        return Python::with_gil(|py| {
            match self.0.call1(py, (path,)) {
                Ok(content) => Ok(content.to_string()),
                Err(_e) => Err(IncludeLoaderError::new(path, ErrorKind::Other))
            }
        });
    }
}

#[pyfunction]
#[pyo3(signature = (input, *, disable_comments=false, social_icon_origin=None, fonts=None, include_loader=None))]
fn mjml2html(
    input: String,
    disable_comments: bool,
    social_icon_origin: Option<String>,
    fonts: Option<HashMap<String, String>>,
    include_loader: Option<PyObject>,
) -> PyResult<String> {
    let parse_opts = ParserOptions {
        include_loader: match include_loader {
            None => Box::new(noop_loader::NoopIncludeLoader),
            Some(item) => Box::new(CallbackIncludeLoader(item)),
        },
    };
    let root = match mrml::parse_with_options(input, &parse_opts) {
        Ok(root) => root,
        Err(e) => return Err(PyValueError::new_err(e.to_string())),
    };

    let render_opts = RenderOptions {
        disable_comments,
        social_icon_origin: match social_icon_origin {
            None => None,
            Some(item) => Some(item.into()),
        },
        fonts: match fonts {
            None => RenderOptions::default().fonts,
            Some(item) => item
                .into_iter()
                .map(|(k, v)| (k.to_string(), Cow::from(v)))
                .collect(),
        },
    };
    return match root.render(&render_opts) {
        Ok(content) => Ok(content),
        Err(e) => Err(PyValueError::new_err(e.to_string())),
    };
}

#[pymodule]
fn mjml(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(mjml2html, m)?)?;
    Ok(())
}
