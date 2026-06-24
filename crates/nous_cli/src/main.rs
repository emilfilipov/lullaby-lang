use std::{env, fs, path::PathBuf, process::ExitCode};

use nous_lexer::{Diagnostic, lex, validate_source_path};
use nous_parser::parse;
use nous_semantics::validate;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        print_help();
        return Ok(());
    };

    match command.as_str() {
        "check" => {
            let Some(path) = args.next() else {
                return Err("usage: nlang check <file.nl>".to_string());
            };
            check(PathBuf::from(path))
        }
        "--version" | "-V" => {
            println!("nlang {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        "--help" | "-h" | "help" => {
            print_help();
            Ok(())
        }
        other => Err(format!("unknown command `{other}`\n\nrun `nlang --help`")),
    }
}

fn check(path: PathBuf) -> Result<(), String> {
    validate_source_path(&path).map_err(format_lex_diagnostic)?;
    let source = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read `{}`: {error}", path.display()))?;

    let tokens = lex(&source).map_err(format_lex_diagnostics)?;
    let program = parse(&tokens).map_err(format_lex_diagnostics)?;
    validate(&program).map_err(|diagnostics| {
        diagnostics
            .into_iter()
            .map(|diagnostic| match diagnostic.function {
                Some(function) => {
                    format!(
                        "{} in `{function}`: {}",
                        diagnostic.code, diagnostic.message
                    )
                }
                None => format!("{}: {}", diagnostic.code, diagnostic.message),
            })
            .collect::<Vec<_>>()
            .join("\n")
    })?;

    println!("ok: {}", path.display());
    Ok(())
}

fn print_help() {
    println!(
        "nlang {}\n\nusage:\n  nlang check <file.nl>\n  nlang --version",
        env!("CARGO_PKG_VERSION")
    );
}

fn format_lex_diagnostics(diagnostics: Vec<Diagnostic>) -> String {
    diagnostics
        .into_iter()
        .map(format_lex_diagnostic)
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_lex_diagnostic(diagnostic: Diagnostic) -> String {
    format!(
        "{} at {}:{}: {}",
        diagnostic.code, diagnostic.span.line, diagnostic.span.column, diagnostic.message
    )
}
