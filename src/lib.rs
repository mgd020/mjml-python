use mrml;
use mrml::prelude::parser::loader::*;
use mrml::prelude::parser::Origin;
use mrml::prelude::parser::*;
use mrml::prelude::render::*;
use pyo3::exceptions::{PyException, PyUserWarning, PyValueError};
use pyo3::prelude::*;
use std::borrow::Cow;
use std::collections::HashMap;
use std::io::ErrorKind;
use std::sync::{Arc, Mutex, MutexGuard};

/// State shared between the include loader and `parse_inner`. MRML can only carry its own
/// error type through the parser, and only reports locations as offsets or line/column.
#[derive(Debug, Default)]
struct LoaderState {
    /// The exception the callback raised.
    error: Option<PyErr>,
    /// The source text returned for each include path, used to map MRML's locations.
    sources: HashMap<String, String>,
}

type SharedLoaderState = Arc<Mutex<LoaderState>>;

fn lock(state: &SharedLoaderState) -> MutexGuard<'_, LoaderState> {
    state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[derive(Debug)]
struct CallbackIncludeLoader {
    callback: Py<PyAny>,
    state: SharedLoaderState,
}

impl IncludeLoader for CallbackIncludeLoader {
    fn resolve(&self, path: &str) -> Result<String, IncludeLoaderError> {
        Python::attach(|py| {
            match self
                .callback
                .call1(py, (path,))
                .and_then(|o| o.extract::<String>(py))
            {
                Ok(source) => {
                    lock(&self.state)
                        .sources
                        .insert(path.to_string(), source.clone());
                    Ok(source)
                }
                Err(e) => {
                    lock(&self.state).error.get_or_insert(e);
                    Err(IncludeLoaderError::new(path, ErrorKind::Other))
                }
            }
        })
    }
}

pyo3::create_exception!(
    mjml,
    MjmlWarning,
    PyUserWarning,
    "A warning MRML recorded while parsing a template.\n\nAttributes: `kind`, `origin`, `start`, `end`."
);

pyo3::create_exception!(
    mjml,
    MjmlError,
    PyValueError,
    "Raised when a template cannot be parsed or rendered.\n\nAttributes: `kind`, `origin`, `start`, `end`, `path`. They are `None` when they do not apply."
);

fn origin_path(origin: &Origin) -> Option<String> {
    match origin {
        Origin::Root => None,
        Origin::Include { path } => Some(path.clone()),
    }
}

/// Build an `MjmlError` carrying the location details MRML reported (if any).
fn mjml_error(
    message: String,
    kind: &str,
    origin: Option<String>,
    span: Option<(usize, usize)>,
    path: Option<String>,
    cause: Option<PyErr>,
) -> PyErr {
    Python::attach(|py| {
        let err = MjmlError::new_err(message);
        let instance = err.value(py);
        let result = instance
            .setattr("kind", kind)
            .and_then(|_| instance.setattr("origin", origin))
            .and_then(|_| instance.setattr("start", span.map(|s| s.0)))
            .and_then(|_| instance.setattr("end", span.map(|s| s.1)))
            .and_then(|_| instance.setattr("path", path));
        match result {
            Ok(()) => {
                err.set_cause(py, cause);
                err
            }
            Err(e) => e,
        }
    })
}

/// The source text of the root template and every include loaded so far, used to turn MRML's
/// locations into offsets that index a Python `str`.
struct Sources<'a> {
    root: &'a str,
    includes: &'a HashMap<String, String>,
}

impl Sources<'_> {
    fn get(&self, origin: Option<&str>) -> Option<&str> {
        match origin {
            None => Some(self.root),
            Some(path) => self.includes.get(path).map(String::as_str),
        }
    }
}

/// MRML reports spans as UTF-8 byte offsets; Python indexes `str` by character.
fn char_span(src: Option<&str>, start: usize, end: usize) -> (usize, usize) {
    let to_char = |byte: usize| match src {
        Some(src) => src.char_indices().take_while(|(i, _)| *i < byte).count(),
        None => byte,
    };
    (to_char(start), to_char(end))
}

/// Character offset of a 1-based `row`/`col` as reported by the HTML parser (rows are counted
/// by `\n`, columns in characters).
fn line_col_offset(src: &str, row: u32, col: u32) -> Option<usize> {
    let mut line_start = 0;
    if row > 1 {
        let mut line = 1;
        line_start = src.chars().enumerate().find_map(|(i, c)| {
            if c == '\n' {
                line += 1;
                (line == row).then_some(i + 1)
            } else {
                None
            }
        })?;
    }
    let offset = line_start + (col.checked_sub(1)? as usize);
    (offset <= src.chars().count()).then_some(offset)
}

fn parse_error(e: &mrml::prelude::parser::Error, sources: &Sources, cause: Option<PyErr>) -> PyErr {
    use mrml::prelude::parser::Error as E;
    let (kind, origin, span) = match e {
        E::UnexpectedElement { origin, position } => {
            ("unexpected-element", Some(origin), Some(position))
        }
        E::UnexpectedToken { origin, position } => {
            ("unexpected-token", Some(origin), Some(position))
        }
        E::MissingAttribute {
            origin, position, ..
        } => ("missing-attribute", Some(origin), Some(position)),
        E::InvalidAttribute { origin, position } => {
            ("invalid-attribute", Some(origin), Some(position))
        }
        E::InvalidFormat { origin, position } => ("invalid-format", Some(origin), Some(position)),
        E::EndOfStream { origin } => ("end-of-stream", Some(origin), None),
        E::SizeLimit { origin } => ("size-limit", Some(origin), None),
        E::ParserError { origin, .. } => ("parser-error", Some(origin), None),
        E::NoRootNode => ("no-root-node", None, None),
        E::IncludeLoaderError {
            origin, position, ..
        } => ("include-loader-error", Some(origin), Some(position)),
    };
    let origin = origin.and_then(origin_path);
    let src = sources.get(origin.as_deref());
    let mut message = e.to_string();
    let mut span = span.map(|s| char_span(src, s.start, s.end));

    if let E::ParserError { source, .. } = e {
        // The HTML parser reports a line/column. They are only reliable for the root
        // template: for includes MRML reports them relative to an internal wrapper.
        if origin.is_none() {
            message = format!("{e}: {source}");
            let pos = source.pos();
            span = src
                .and_then(|src| line_col_offset(src, pos.row, pos.col))
                .map(|offset| (offset, offset));
        }
    }

    let cause = cause.filter(|_| matches!(e, E::IncludeLoaderError { .. }));
    let mut path = None;
    if let E::IncludeLoaderError { source, .. } = e {
        // `mj-include` whose `include_loader` callback failed: name the path, and
        // chain the callback's exception.
        let origin_name = match &origin {
            None => "root template".to_string(),
            Some(origin) => format!("template from {origin:?}"),
        };
        message = match span {
            Some((start, end)) => format!(
                "unable to load included template {:?} in {origin_name} at position {start}:{end}",
                source.path
            ),
            None => format!(
                "unable to load included template {:?} in {origin_name}",
                source.path
            ),
        };
        path = Some(source.path.clone());
    }

    mjml_error(message, kind, origin, span, path, cause)
}

fn render_error(e: &mrml::prelude::render::Error) -> PyErr {
    use mrml::prelude::render::Error as E;
    let kind = match e {
        E::UnknownFragment(_) => "unknown-fragment",
        E::Format(_) => "format",
        E::InlineCSS(_) => "inline-css",
    };
    mjml_error(e.to_string(), kind, None, None, None, None)
}

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
        let instance = MjmlWarning::new_err(self.message())
            .into_value(py)
            .into_bound(py);
        instance.setattr("kind", &self.kind)?;
        instance.setattr("origin", &self.origin)?;
        instance.setattr("start", self.start)?;
        instance.setattr("end", self.end)?;
        Ok(instance.into_any())
    }
}

impl WarningData {
    fn new(w: &mrml::prelude::parser::Warning, sources: &Sources) -> Self {
        let origin = origin_path(&w.origin);
        let (start, end) = char_span(sources.get(origin.as_deref()), w.span.start, w.span.end);
        WarningData {
            kind: w.kind.as_str().to_string(),
            origin,
            start,
            end,
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
            .map_err(|e| render_error(&e))
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

fn is_exception(err: &PyErr) -> bool {
    Python::attach(|py| err.is_instance_of::<PyException>(py))
}

fn parse_inner(input: String, include_loader: Option<Py<PyAny>>) -> PyResult<Mjml> {
    let state = SharedLoaderState::default();
    let parse_opts = ParserOptions {
        include_loader: match include_loader {
            None => Box::new(noop_loader::NoopIncludeLoader),
            Some(callback) => Box::new(CallbackIncludeLoader {
                callback,
                state: state.clone(),
            }),
        },
    };

    let result = mrml::parse_with_options(&input, &parse_opts);
    let mut guard = lock(&state);
    let LoaderState { error, sources } = &mut *guard;
    let sources = Sources {
        root: &input,
        includes: sources,
    };
    match result {
        Ok(output) => Ok(Mjml {
            warnings: output
                .warnings
                .iter()
                .map(|w| WarningData::new(w, &sources))
                .collect(),
            element: output.element,
        }),
        Err(e) => Err(match error.take() {
            // `KeyboardInterrupt`, `SystemExit` and the like are not template errors.
            Some(raised) if !is_exception(&raised) => raised,
            raised => parse_error(&e, &sources, raised),
        }),
    }
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
    m.add("MjmlError", m.py().get_type::<MjmlError>())?;
    Ok(())
}
