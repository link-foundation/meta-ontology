//! `meta-ontology` CLI binary.
//!
//! Subcommands:
//! - `list`         — print every concept name, alphabetical
//! - `show <name>`  — print a concept's label, definitions, mappings
//! - `check-words <path>` — fail if any word in `path` is unknown
//! - `langs`        — list languages with declared exponents

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Subcommand;
use lino_arguments::Parser;

use meta_ontology::loader::{load_default, load_from_dir};
use meta_ontology::words::scan_path;

#[derive(Parser, Debug)]
#[command(name = "meta-ontology", about = "Meta-ontology library + CLI", version)]
struct Cli {
    /// Override the data directory (defaults to <crate>/data).
    #[arg(long, env = "META_ONTOLOGY_DATA")]
    data: Option<PathBuf>,

    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// List every concept name, alphabetical.
    List,
    /// Print a concept's label, definitions, and mappings.
    Show { name: String },
    /// Fail (exit 1) if any human-language word in `path` is not in the
    /// ontology and not on the allow-list.
    CheckWords { path: PathBuf },
    /// List languages with declared exponents.
    Langs,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let ontology = cli.data.as_ref().map_or_else(load_default, load_from_dir);
    let ontology = match ontology {
        Ok(o) => o,
        Err(e) => {
            eprintln!("error loading ontology: {e}");
            return ExitCode::from(1);
        }
    };

    match cli.cmd {
        Cmd::List => {
            for name in ontology.names() {
                println!("{name}");
            }
            ExitCode::SUCCESS
        }
        Cmd::Show { name } => {
            let Some(c) = ontology.find(&name) else {
                eprintln!("unknown concept: {name}");
                return ExitCode::from(1);
            };
            println!("name:     {}", c.name);
            if !c.label.is_empty() {
                println!("label:    {}", c.label);
            }
            if !c.origin.is_empty() {
                println!("origin:   {}", c.origin);
            }
            if !c.category.is_empty() {
                println!("category: {}", c.category);
            }
            if !c.allolexes.is_empty() {
                println!("allolex:  {}", c.allolexes.join(", "));
            }
            if !c.definitions.is_empty() {
                println!("definitions:");
                for def in &c.definitions {
                    println!("  - {}", def.words.join(" "));
                }
            }
            if !c.mappings.is_empty() {
                println!("mappings:");
                for m in &c.mappings {
                    println!("  - {} {}", m.kind, m.external);
                }
            }
            if !c.exponents.is_empty() {
                println!("exponents:");
                for (lang, form) in &c.exponents {
                    println!("  - {lang}: {form}");
                }
            }
            ExitCode::SUCCESS
        }
        Cmd::CheckWords { path } => match scan_path(&path, &ontology) {
            Ok(unknown) => {
                if unknown.is_empty() {
                    println!("ok: every word in {} is covered", path.display());
                    ExitCode::SUCCESS
                } else {
                    eprintln!("{} unknown words found:", unknown.len());
                    for u in &unknown {
                        eprintln!("  - {:?} in {}:{}", u.word, u.file.display(), u.line);
                    }
                    ExitCode::from(1)
                }
            }
            Err(e) => {
                eprintln!("error scanning {}: {e}", path.display());
                ExitCode::from(1)
            }
        },
        Cmd::Langs => {
            for code in ontology.languages() {
                println!("{code}");
            }
            ExitCode::SUCCESS
        }
    }
}
