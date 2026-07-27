//! `meta-ontology` CLI binary.
//!
//! Subcommands:
//! - `list`         — print every concept name, alphabetical
//! - `show <name>`  — print a concept's label, definitions, mappings
//! - `check-words <path>` — fail if any word in `path` is unknown
//! - `langs`        — list languages with declared exponents
//! - `validate`     — validate schema, identity, provenance, and relationships
//! - `export-json`  — emit the versioned public interchange model
//! - `json-schema`  — emit the generated interchange JSON Schema
//! - `plan-import`  — validate JSON and emit a side-effect-free change plan
//! - `search <query>` — search catalog identity and descriptive fields

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Subcommand, ValueEnum};
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
    /// Validate all catalog contract layers.
    Validate {
        /// Diagnostic output shape.
        #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
        format: OutputFormat,
    },
    /// Export the versioned public model as JSON.
    ExportJson,
    /// Export the generated interchange JSON Schema.
    JsonSchema,
    /// Validate a JSON document and emit its side-effect-free import plan.
    PlanImport {
        /// Candidate full-replacement interchange document.
        path: PathBuf,
        /// Change report output shape.
        #[arg(long, value_enum, default_value_t = PlanFormat::Human)]
        format: PlanFormat,
    },
    /// Search IDs, names, labels, aliases, and definitions.
    Search {
        /// Case-insensitive search text.
        query: String,
        /// Maximum number of deterministic results.
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum OutputFormat {
    Human,
    Json,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum PlanFormat {
    Human,
    Json,
    Ndjson,
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
        Cmd::Validate { format } => {
            let report = ontology.validate();
            match format {
                OutputFormat::Human => {
                    if report.diagnostics().is_empty() {
                        println!(
                            "valid: schema v{}, {} concepts, {} relationships",
                            ontology.schema_version(),
                            ontology.len(),
                            ontology.relationships().count()
                        );
                    } else {
                        for diagnostic in report.diagnostics() {
                            let location =
                                diagnostic
                                    .source_uri
                                    .as_deref()
                                    .map_or_else(String::new, |uri| {
                                        diagnostic.source_line.map_or_else(
                                            || format!(" ({uri})"),
                                            |line| format!(" ({uri}:{line})"),
                                        )
                                    });
                            eprintln!(
                                "{} {:?}: {}{}",
                                diagnostic.code, diagnostic.severity, diagnostic.message, location
                            );
                        }
                    }
                }
                OutputFormat::Json => match serde_json::to_string_pretty(&report) {
                    Ok(json) => println!("{json}"),
                    Err(error) => {
                        eprintln!("error serializing validation report: {error}");
                        return ExitCode::from(1);
                    }
                },
            }
            if report.is_valid() {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Cmd::ExportJson => match ontology.to_json_pretty() {
            Ok(json) => {
                println!("{json}");
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("error exporting ontology: {error}");
                ExitCode::from(1)
            }
        },
        Cmd::JsonSchema => match meta_ontology::Ontology::json_schema_pretty() {
            Ok(json) => {
                println!("{json}");
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("error exporting JSON Schema: {error}");
                ExitCode::from(1)
            }
        },
        Cmd::PlanImport { path, format } => {
            let json = match std::fs::read_to_string(&path) {
                Ok(json) => json,
                Err(error) => {
                    eprintln!("error reading import {}: {error}", path.display());
                    return ExitCode::from(1);
                }
            };
            let candidate = match meta_ontology::Ontology::from_json(&json) {
                Ok(candidate) => candidate,
                Err(error) => {
                    eprintln!("error validating import {}: {error}", path.display());
                    return ExitCode::from(1);
                }
            };
            let plan = ontology.plan_import(&candidate.to_interchange());
            match format {
                PlanFormat::Human => {
                    for change in plan.changes() {
                        println!("{:?}\t{:?}\t{}", change.kind, change.object_kind, change.id);
                    }
                }
                PlanFormat::Json => match serde_json::to_string_pretty(&plan) {
                    Ok(json) => println!("{json}"),
                    Err(error) => {
                        eprintln!("error serializing import plan: {error}");
                        return ExitCode::from(1);
                    }
                },
                PlanFormat::Ndjson => match plan.to_ndjson() {
                    Ok(json) => print!("{json}"),
                    Err(error) => {
                        eprintln!("error serializing import plan: {error}");
                        return ExitCode::from(1);
                    }
                },
            }
            ExitCode::SUCCESS
        }
        Cmd::Search { query, limit } => {
            for result in ontology.search(&query, limit) {
                println!(
                    "{}\t{}\t{}\t{}",
                    result.score, result.concept.id, result.concept.name, result.concept.label
                );
            }
            ExitCode::SUCCESS
        }
    }
}
