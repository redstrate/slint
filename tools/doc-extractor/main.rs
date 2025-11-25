// Copyright © SixtyFPS GmbH <info@slint.dev>
// SPDX-License-Identifier: GPL-3.0-only OR LicenseRef-Slint-Royalty-free-2.0 OR LicenseRef-Slint-Software-3.0

use clap::Parser;
use i_slint_compiler::diagnostics::{BuildDiagnostics, Spanned};
use i_slint_compiler::parser::{syntax_nodes, SyntaxKind, SyntaxNode};
use smol_str::SmolStr;
use std::fmt::Write;

#[derive(clap::Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(name = "path to .slint file(s)", action)]
    paths: Vec<std::path::PathBuf>,

    #[arg(long = "default-domain", short = 'd')]
    domain: Option<String>,

    #[arg(
        name = "file",
        short = 'o',
        help = "Write output to specified file (instead of messages.md)."
    )]
    output: Option<std::path::PathBuf>,

    //#[arg(long = "omit-header", help = r#"Don’t write header with ‘msgid ""’ entry"#)]
    //omit_header: bool,
    //
    //#[arg(long = "copyright-holder", help = "Set the copyright holder in the output")]
    //copyright_holder: Option<String>,
    //
    #[arg(long = "package-name", help = "Set the package name in the header of the output")]
    package_name: Option<String>,

    #[arg(long = "package-version", help = "Set the package version in the header of the output")]
    package_version: Option<String>,
    //
    // #[arg(
    //     long = "msgid-bugs-address",
    //     help = "Set the reporting address for msgid bugs. This is the email address or URL to which the translators shall report bugs in the untranslated strings"
    // )]
    // msgid_bugs_address: Option<String>,
    #[arg(long = "join-existing", short = 'j')]
    /// Join messages with existing file
    join_existing: bool,
}

fn main() -> std::io::Result<()> {
    let args = Cli::parse();

    let output = args
        .output
        .unwrap_or_else(|| format!("{}.md", args.domain.as_deref().unwrap_or("messages")).into());

    let mut messages = String::new();

    for path in args.paths {
        process_file(path, &mut messages)?
    }

    std::fs::write(&output, &messages)?;
    Ok(())
}

fn process_file(path: std::path::PathBuf, messages: &mut String) -> std::io::Result<()> {
    let mut diag = BuildDiagnostics::default();
    let syntax_node = i_slint_compiler::parser::parse_file(path, &mut diag).ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::Other, diag.to_string_vec().join(", "))
    })?;
    export_visit_node(syntax_node.clone(), messages);
    visit_node(syntax_node, messages, None);

    Ok(())
}

fn visit_node(node: SyntaxNode, results: &mut String, current_context: Option<SmolStr>) {
    for n in node.children() {
        if n.kind() == SyntaxKind::Component {
            let component = syntax_nodes::Component::new(n.clone());
            let comment = get_comments_before_line(n.first_token().unwrap());

            results.push_str("## ");
            results.push_str(&i_slint_compiler::parser::normalize_identifier(&component.unwrap().DeclaredIdentifier().child_text(SyntaxKind::Identifier).unwrap()).to_string());
            results.push('\n');
            if let Some(comment) = comment {
                results.push_str(&comment);
                results.push('\n');
            }
            results.push('\n');
        } else if n.kind() == SyntaxKind::PropertyDeclaration {
            let component = syntax_nodes::PropertyDeclaration::new(n.clone());
            let comment = get_comments_before_line(n.first_token().unwrap());

            results.push_str("### ");
            results.push_str(&i_slint_compiler::parser::normalize_identifier(&component.unwrap().DeclaredIdentifier().child_text(SyntaxKind::Identifier).unwrap()).to_string());

            results.push_str(": `");

            let component = syntax_nodes::PropertyDeclaration::new(n.clone());
            results.push_str(&i_slint_compiler::parser::normalize_identifier(&component.unwrap().Type().unwrap().text().to_string()));

            results.push_str("`\n");

            if let Some(comment) = comment {
                results.push_str(&comment);
                results.push('\n');
            }
        }

        let current_context = syntax_nodes::Component::new(n.clone())
            .and_then(|x| {
                x.DeclaredIdentifier()
                    .child_text(SyntaxKind::Identifier)
                    .map(|t| i_slint_compiler::parser::normalize_identifier(&t))
            })
            .or_else(|| current_context.clone());
        visit_node(n, results, current_context);
    }
}

fn get_comments_before_line(token: i_slint_compiler::parser::SyntaxToken) -> Option<String> {
    let mut token = token.prev_token()?;
    token = token.prev_token()?; // Eat the whitespace
    if token.kind() == SyntaxKind::Comment {
        Some(token.text().to_string())
    } else {
        None
    }
}

fn export_visit_node(node: SyntaxNode, results: &mut String) {
    for n in node.children() {
        if n.kind() == SyntaxKind::ExportsList {
            let component = syntax_nodes::ExportsList::new(n.clone());
            dbg!(component.unwrap().first_token());
        }

        export_visit_node(n, results);
    }
}
