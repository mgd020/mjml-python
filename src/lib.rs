use mrml;
use mrml::prelude::parser::loader::*;
use mrml::prelude::parser::Origin;
use mrml::prelude::parser::*;
use mrml::prelude::render::*;
use pyo3::exceptions::{PyUserWarning, PyValueError};
use pyo3::prelude::*;
use std::borrow::Cow;
use std::collections::HashMap;
use std::io::ErrorKind;

#[derive(Debug)]
struct CallbackIncludeLoader(pub Py<PyAny>);

impl IncludeLoader for CallbackIncludeLoader {
    fn resolve(&self, path: &str) -> Result<String, IncludeLoaderError> {
        Python::attach(|py| {
            self.0
                .call1(py, (path,))
                .and_then(|o| o.extract::<String>(py))
                .map_err(|_| IncludeLoaderError::new(path, ErrorKind::Other))
        })
    }
}

pyo3::create_exception!(
    mjml,
    MjmlWarning,
    PyUserWarning,
    "A warning MRML recorded while parsing a template.\n\nAttributes: `kind`, `origin`, `start`, `end`."
);

/// Plain-data copy of an MRML warning, turned into an `MjmlWarning` instance on demand.
#[derive(Clone, Debug)]
struct WarningData {
    kind: String,
    origin: Option<String>,
    start: usize,
    end: usize,
}

impl WarningData {
    fn message(&self) -> String {
        let origin = match &self.origin {
            None => "root template".to_string(),
            Some(path) => format!("include {path:?}"),
        };
        format!("{} at {}:{} in {}", self.kind, self.start, self.end, origin)
    }

    fn to_py<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let instance = MjmlWarning::new_err(self.message()).into_value(py).into_bound(py);
        instance.setattr("kind", &self.kind)?;
        instance.setattr("origin", &self.origin)?;
        instance.setattr("start", self.start)?;
        instance.setattr("end", self.end)?;
        Ok(instance.into_any())
    }
}

impl From<&mrml::prelude::parser::Warning> for WarningData {
    fn from(w: &mrml::prelude::parser::Warning) -> Self {
        WarningData {
            kind: w.kind.as_str().to_string(),
            origin: match &w.origin {
                Origin::Root => None,
                Origin::Include { path } => Some(path.clone()),
            },
            start: w.span.start,
            end: w.span.end,
        }
    }
}

/// A parsed MJML template.
#[pyclass(module = "mjml", frozen)]
struct Mjml {
    element: mrml::mjml::Mjml,
    warnings: Vec<WarningData>,
}

impl Mjml {
    fn render_inner(
        &self,
        disable_comments: bool,
        social_icon_origin: Option<String>,
        fonts: Option<HashMap<String, String>>,
    ) -> PyResult<String> {
        let fonts_map = match fonts {
            None => RenderOptions::default().fonts,
            Some(item) => item.into_iter().map(|(k, v)| (k, Cow::Owned(v))).collect(),
        };

        let render_opts = RenderOptions {
            disable_comments,
            social_icon_origin: social_icon_origin.map(Into::into),
            fonts: fonts_map,
        };

        self.element
            .render(&render_opts)
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }
}

#[pymethods]
impl Mjml {
    /// The contents of `<mj-title>`, if any.
    #[getter]
    fn title(&self) -> Option<String> {
        self.element.get_title()
    }

    /// The contents of `<mj-preview>`, if any.
    #[getter]
    fn preview(&self) -> Option<String> {
        self.element.get_preview()
    }

    /// Warnings recorded while parsing.
    #[getter]
    fn warnings<'py>(&self, py: Python<'py>) -> PyResult<Vec<Bound<'py, PyAny>>> {
        self.warnings.iter().map(|w| w.to_py(py)).collect()
    }

    #[pyo3(signature = (*, disable_comments=false, social_icon_origin=None, fonts=None))]
    fn render(
        &self,
        py: Python<'_>,
        disable_comments: bool,
        social_icon_origin: Option<String>,
        fonts: Option<HashMap<String, String>>,
    ) -> PyResult<String> {
        // Release the GIL while rendering so other Python threads can run.
        py.detach(|| self.render_inner(disable_comments, social_icon_origin, fonts))
    }
}

fn parse_inner(input: String, include_loader: Option<Py<PyAny>>) -> PyResult<Mjml> {
    let parse_opts = ParserOptions {
        include_loader: match include_loader {
            None => Box::new(noop_loader::NoopIncludeLoader),
            Some(item) => Box::new(CallbackIncludeLoader(item)),
        },
    };

    let output = mrml::parse_with_options(input, &parse_opts)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;

    Ok(Mjml {
        warnings: output.warnings.iter().map(WarningData::from).collect(),
        element: output.element,
    })
}

#[pyfunction]
#[pyo3(signature = (input, *, include_loader=None))]
fn parse(py: Python<'_>, input: String, include_loader: Option<Py<PyAny>>) -> PyResult<Mjml> {
    // Release the GIL while parsing so other Python threads can run. The
    // include loader callback re-attaches via `Python::attach`.
    py.detach(move || parse_inner(input, include_loader))
}

#[pyfunction]
#[pyo3(signature = (input, *, disable_comments=false, social_icon_origin=None, fonts=None, include_loader=None))]
fn mjml2html(
    py: Python<'_>,
    input: String,
    disable_comments: bool,
    social_icon_origin: Option<String>,
    fonts: Option<HashMap<String, String>>,
    include_loader: Option<Py<PyAny>>,
) -> PyResult<String> {
    // Release the GIL while parsing and rendering so other Python threads can
    // run. The include loader callback re-attaches via `Python::attach`.
    let parsed = py.detach(move || parse_inner(input, include_loader))?;
    // Emitted with the GIL held. If the caller turned warnings into errors,
    // this raises before rendering.
    if !parsed.warnings.is_empty() {
        let warn = py.import("warnings")?.getattr("warn")?;
        for warning in &parsed.warnings {
            warn.call1((warning.to_py(py)?, py.None(), 1))?;
        }
    }
    py.detach(move || parsed.render_inner(disable_comments, social_icon_origin, fonts))
}

#[pymodule]
fn mjml(_py: Python<'_>, m: &Bound<PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(mjml2html, m)?)?;
    m.add_function(wrap_pyfunction!(parse, m)?)?;
    m.add_class::<Mjml>()?;
    m.add("MjmlWarning", m.py().get_type::<MjmlWarning>())?;
    Ok(())
}
