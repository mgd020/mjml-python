use std::borrow::Cow;
use std::collections::HashMap;
use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;
use mrml;

#[pyfunction]
#[pyo3(signature = (input, *, disable_comments=false, social_icon_origin=None, fonts=None))]
fn mjml2html(
    input: String,
    disable_comments: bool,
    social_icon_origin: Option<String>,
    fonts: Option<HashMap<String, String>>
) -> PyResult<String> {
    let root = match mrml::parse(input) {
        Ok(root) => root,
        Err(e) => return Err(PyValueError::new_err(e.to_string())),
    };

    let default_opts = mrml::prelude::render::Options::default();
    let opts = mrml::prelude::render::Options {
        disable_comments,
        social_icon_origin: match social_icon_origin {
            None => default_opts.social_icon_origin,
            Some(item) => Some(item.into())
        },
        fonts: match fonts {
            None => default_opts.fonts,
            Some(item) => item
                .into_iter()
                .map(|(k, v)| (k.to_string(), Cow::from(v)))
                .collect()
        },
    };
    
    return match root.render(&opts) {
        Ok(content) => Ok(content),
        Err(e) => Err(PyValueError::new_err(e.to_string()))
    };
}

#[pymodule]
fn mjml(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(mjml2html, m)?)?;
    Ok(())
}

