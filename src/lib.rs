use mrml;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

#[pyclass]
struct MJML {
    root: mrml::mjml::MJML,
    template: String,
}

#[pymethods]
impl MJML {
    #[staticmethod]
    fn parse(input: String) -> PyResult<MJML> {
        let template = input.clone();
        return match mrml::parse(input) {
            Ok(root) => Ok(MJML {
                root,
                template,
            }),
            Err(e) => return Err(PyValueError::new_err(e.to_string())),
        };
    }

    fn render(&self) -> PyResult<String> {
        let opts = mrml::prelude::render::Options::default();
        return match self.root.render(&opts) {
            Ok(content) => Ok(content),
            Err(e) => Err(PyValueError::new_err(e.to_string())),
        };
    }

    #[getter]
    fn title(&self) -> PyResult<String> {
        return match self.root.get_title() {
            Some(title) => Ok(title),
            None => Err(PyValueError::new_err("No title found")),
        };
    }

    #[getter]
    fn preview(&self) -> PyResult<String> {
        return match self.root.get_preview() {
            Some(preview) => Ok(preview),
            None => Err(PyValueError::new_err("No preview found")),
        };
    }

    #[getter]
    fn template(&self) -> PyResult<String> {
        return Ok(self.template.clone());
    }
}

#[pyfunction]
fn parse(input: String) -> PyResult<MJML> {
    return MJML::parse(input);
}

#[pyfunction]
fn mjml2html(input: String) -> PyResult<String> {
    let root = MJML::parse(input)?;
    return root.render();
}

#[pymodule]
fn mjml(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<MJML>()?;
    m.add_function(wrap_pyfunction!(mjml2html, m)?)?;
    m.add_function(wrap_pyfunction!(parse, m)?)?;
    Ok(())
}
