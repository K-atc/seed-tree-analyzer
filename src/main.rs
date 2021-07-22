pub mod mutation_graph;

extern crate clap;
use clap::{App, Arg, SubCommand};
extern crate regex;

use crate::mutation_graph::perser::parse_mutation_graph_file;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

fn main() {
    let matches =
        App::new("libfuzzer-mutation-graph-tool")
            .version("1.0")
            .author("Nao Tomori (@K_atc)")
            .about("A Tool to interact wth libfuzzer's mutation graph file")
            .subcommand(SubCommand::with_name("parse").about("Just parse mutation graph file"))
            .subcommand(
                SubCommand::with_name("pred")
                    .about("List predecessor of given node")
                    .arg(
                        Arg::with_name("SHA1")
                            .help("SHA1 (a node name; i.e. seed file name)")
                            .required(true)
                            .index(1),
                    ),
            )
            .subcommand(SubCommand::with_name("plot").about(
                "Plot mutation graph file and save as PNG, SVG. This command requires graphviz",
            ))
            .arg(
                Arg::with_name("FILE")
                    .help("A mutation graph file")
                    .required(true)
                    .index(1),
            )
            .get_matches();

    let mutation_graph_file = Path::new(matches.value_of("FILE").unwrap());

    let graph = match parse_mutation_graph_file(mutation_graph_file) {
        Ok(graph) => graph,
        Err(why) => {
            eprintln!(
                "[!] Failed to parse file {:?}: {:?}",
                mutation_graph_file, why
            );
            return;
        }
    };

    if let Some(_matches) = matches.subcommand_matches("parse") {
        println!("{:#?}", graph);
    } else if let Some(matches) = matches.subcommand_matches("pred") {
        let node = match matches.value_of("SHA1") {
            Some(node) => node.to_string(),
            None => {
                eprintln!("[!] SHA1 is not specified");
                return;
            }
        };

        match graph.predecessors_of(&node) {
            Ok(predecessors) => {
                if predecessors.len() > 0 {
                    for name in predecessors.iter() {
                        println!("{}", name)
                    }
                } else {
                    eprintln!("[*] Given node does not have predecessors: sha1={}", node);
                }
            }
            Err(why) => {
                eprintln!("[!] Failed to get predecessors of {}: {:?}", node, why)
            }
        }
    } else if let Some(_matches) = matches.subcommand_matches("plot") {
        let dot_graph_text = graph.dot_graph().expect("Failed to generate dot file");

        plot_dot_graph(&dot_graph_text, "png", &mutation_graph_file);
        plot_dot_graph(&dot_graph_text, "svg", &mutation_graph_file);
    } else {
        eprintln!("[!] No subcommand specified")
    }
}

fn plot_dot_graph(dot_graph_text: &String, format: &'static str, original_file: &Path) {
    let mut child = Command::new("dot")
        .arg(format!("-T{}", format))
        .arg("-o")
        .arg(original_file.with_extension(format).as_os_str())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to spawn \"dot\" (graphviz)");

    {
        let mut stdin = child.stdin.take().expect("Failed to open stdin");
        stdin
            .write_all(dot_graph_text.as_bytes())
            .expect("Failed to write to stdin");
        // Drop `stdin` to close stdin
    }

    let _ = child.wait_with_output().expect("Failed to read stdout");
}
