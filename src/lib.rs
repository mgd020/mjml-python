use clap::{Parser, ValueHint};
use clap_verbosity_flag::Verbosity;
use mrml::mjml::Mjml;
use mrml::prelude::parser::loader::*;
use mrml::prelude::parser::*;
use mrml::prelude::print::Print;
use mrml::prelude::render::RenderOptions;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::ErrorKind;
use std::path::PathBuf;

#[derive(Debug)]
struct CallbackIncludeLoader(pub PyObject);

impl IncludeLoader for CallbackIncludeLoader {
    fn resolve(&self, path: &str) -> Result<String, IncludeLoaderError> {
        Python::with_gil(|py| match self.0.call1(py, (path,)) {
            Ok(content) => Ok(content.to_string()),
            Err(_e) => Err(IncludeLoaderError::new(path, ErrorKind::Other)),
        })
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
        social_icon_origin: social_icon_origin.map(|item| item.into()),
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

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to your mjml file
    #[arg(index = 1, value_hint = ValueHint::FilePath,)]
    pub input: Vec<String>,

    /// Migratie the input
    #[arg(short, long, default_value = "false")]
    pub migrate: bool,

    /// Certify(Validate) the input
    #[arg(short, long, default_value = "false")]
    pub certify: bool,

    /// Render and redirect to stdout
    #[arg(short, long, default_value = "false")]
    pub stdout: bool,

    /// Render and redirect to file
    #[arg(short, long)]
    pub output: Option<String>,

    /// Watch for changes and re-render
    #[arg(short, long, default_value = "false")]
    pub watch: bool,

    /// Render options
    #[command(flatten)]
    render: Render,

    #[command(flatten)]
    verbose: Verbosity,
}

#[derive(Debug, Parser, Clone)]
struct Render {
    /// Remove comments from html output
    #[arg(long, required_if_eq_all=[("migrate", "false"), ("certify", "false")])]
    pub disable_comments: bool,
    /// Base url for social icons
    #[clap(long,required_if_eq_all=[("migrate", "false"), ("certify", "false")])]
    pub social_icon_origin: Option<String>,
}

impl From<Render> for RenderOptions {
    fn from(value: Render) -> Self {
        Self {
            disable_comments: value.disable_comments,
            social_icon_origin: value.social_icon_origin.map(Cow::Owned),
            ..Default::default()
        }
    }
}

fn parse_input_file(input_path: &PathBuf) -> Result<String, std::io::Error> {
    let mut input_file = File::open(input_path)?;
    let mut input_contents = String::new();
    input_file.read_to_string(&mut input_contents)?;
    Ok(input_contents)
}

fn parse_mjml(input_contents: &str) -> Result<Mjml, Error> {
    Mjml::parse(input_contents)
}

fn create_and_write_file(path: &str, output: &str) {
    match File::create(path) {
        Ok(mut file) => match file.write_all(output.as_bytes()) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Error writing to output file: {}", e);
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("Error creating output file: {}", e);
            std::process::exit(1);
        }
    }
}

fn process_cli_args(cli: &Cli) {
    let input_contents = match cli.input.len() {
        1 => {
            eprintln!("❌ No input file provided");
            std::process::exit(1);
        }
        2 => parse_input_file(&PathBuf::from(&cli.input[1])).expect("❌ Error reading input file"),
        _ => {
            eprintln!("❌ Only one input file is allowed");
            std::process::exit(1);
        }
    };

    let root = match parse_mjml(&input_contents) {
        Ok(root) => root,
        Err(e) => {
            eprintln!("❌ Error parsing input file: {}", e);
            std::process::exit(1);
        }
    };
    if cli.certify {
        println!("⭐⭐⭐ Input file is valid mjml");
        std::process::exit(0);
    }
    let output = root.print(true, 0, 2);
    // remove blank lines
    let output = output
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<&str>>()
        .join("\n");
    if cli.migrate {
        let output_file = match &cli.output {
            None => cli.input[1].clone(),
            Some(output_path) => output_path.clone(),
        };
        create_and_write_file(&output_file, &output);
        if cli.stdout {
            println!("{}", output);
        }
        std::process::exit(0);
    }

    let render_opts: RenderOptions = cli.render.clone().into();
    let rendered = root
        .render(&render_opts)
        .expect("❌ Error rendering input file");
    let output_file = match &cli.output {
        None => cli.input[1].clone().replace(".mjml", ".html"),
        Some(output_path) => output_path.clone(),
    };
    create_and_write_file(&output_file, &rendered);
    if cli.stdout {
        println!("{}", rendered);
    }
}

#[pyfunction]
fn run_cli() {
    let cli = Cli::parse();
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .init();
    process_cli_args(&cli);
}

#[pymodule]
fn mjml(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(mjml2html, m)?)?;
    m.add_function(wrap_pyfunction!(run_cli, m)?)?;
    Ok(())
}
