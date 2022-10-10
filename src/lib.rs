use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;
use mrml;

#[pyfunction]
fn mjml2html(input: String) -> PyResult<String> {
    let root = match mrml::parse(input) {
        Ok(root) => root,
        Err(e) => return Err(PyValueError::new_err(e.to_string())),
    };
    let opts = mrml::prelude::render::Options::default();
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
