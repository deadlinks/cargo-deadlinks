use cargo_deadlinks::IgnoredFile;
use log::LevelFilter;
use pico_args::Error;
use std::{
    fmt::{self, Display},
    path::PathBuf,
};
use toml::Value;

pub fn parse_ignore_file(
    path: Option<PathBuf>,
) -> Result<(Vec<IgnoredFile>, Vec<IgnoredFile>), Box<dyn std::error::Error>> {
    let is_required = path.is_some();
    let path = path.unwrap_or_else(|| "deadlinks.toml".into());

    let contents = match std::fs::read_to_string(path) {
        Ok(toml) => toml,
        Err(err) => {
            return if is_required {
                Err(err.into())
            } else {
                // We proactively looked for `deadlinks.toml`, but it wasn't there.
                // Pretend it was an empty file.
                Ok((vec![], vec![]))
            };
        }
    };
    let val: Value = contents.parse()?;
    let ignores = match val {
        // top-level should always be a table
        Value::Table(values) => values,
        _ => return Err(format!(
            "invalid TOML format: expected a top-level table, got {:?}",
            val
        ).into()),
    };

    // [(fragment | intra_doc_links)] key = [a, b, c]
    let parse = |val: Value, files: &mut Vec<_>| -> Result<_, Box<dyn std::error::Error>> {
        let map = match val {
            Value::Table(map) => map,
            _ => return Err(format!(
                "invalid TOML format: expected a table, got {:?}",
                val
            ).into()),
        };
        for (file, val) in map {
            let links = match val {
                Value::Array(links) => links,
                _ => return Err(format!(
                    "invalid TOML format: expected a list of links, got {:?}",
                    val
                ).into()),
            };
            let links = links
                .into_iter()
                .map(|val| match val {
                    Value::String(link) => Ok(link),
                    _ => Err(format!(
                        "invalid TOML format: expected a string, got {:?}",
                        val
                    )),
                })
                .collect::<Result<_, _>>()?;
            files.push(IgnoredFile {
                path: PathBuf::from(file),
                links,
            });
        }
        Ok(())
    };

    let (mut ignored_links, mut ignored_intra_doc_links) = (vec![], vec![]);
    for (key, val) in ignores {
        if key == "fragments" {
            parse(val, &mut ignored_links)
        } else if key == "intra_doc_links" {
            parse(val, &mut ignored_intra_doc_links)
        } else {
            Err(format!(
                "invalid TOML format: expected 'fragments' or 'intra_doc_links', got {}",
                key
            )
            .into())
        }?
    }

    Ok((ignored_links, ignored_intra_doc_links))
}

/// Initalizes the logger according to the provided config flags.
pub fn init_logger(debug: bool, verbose: bool, krate: &str) {
    let mut builder = env_logger::Builder::new();
    match (debug, verbose) {
        (true, _) => {
            builder.filter(Some(krate), LevelFilter::Debug);
        }
        (false, true) => {
            builder.filter(Some(krate), LevelFilter::Info);
        }
        _ => {}
    }
    builder.parse_default_env().init();
}

// See https://github.com/RazrFalcon/pico-args/pull/26
pub struct PicoError(pub Error);

impl Display for PicoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.0 {
            Error::ArgumentParsingFailed { cause } => {
                write!(f, "failed to parse arguments: {}", cause)
            }
            Error::Utf8ArgumentParsingFailed { value, cause } => {
                write!(f, "failed to parse '{}': {}", value, cause)
            }
            _ => self.0.fmt(f),
        }
    }
}

impl From<Error> for PicoError {
    fn from(err: Error) -> Self {
        Self(err)
    }
}
