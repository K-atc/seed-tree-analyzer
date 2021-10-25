use crate::seed_tree::error::MutationGraphError;
use crate::seed_tree::plot_options::plot_option::PlotOption;
use crate::seed_tree::plot_options::PlotOptions;
use crate::seed_tree::sha1_string::Sha1String;
use crate::seed_tree::MutationGraph;
use clap::ArgMatches;
use std::collections::HashSet;
use std::iter::FromIterator;

pub(crate) fn filter(matches: &ArgMatches, graph: &MutationGraph, plot_options: &[PlotOption]) {
    let predecessors = match matches.value_of("PRED_ID") {
        Some(node) => Some(Sha1String::from(node)),
        None => None,
    };
    log::info!("predecessors = {:?}", predecessors);

    let leaves = matches.is_present("leaves");
    log::info!("leaves = {:?}", leaves);

    let filtered_graph = match do_filter(graph, predecessors, leaves) {
        Ok(graph) => graph,
        Err(why) => panic!("Failed to filter seed tree: {:?}", why),
    };

    match filtered_graph.dot_graph(PlotOptions::from(plot_options).unwrap()) {
        Ok(graph) => println!("{}", graph),
        Err(why) => panic!("Failed to convert to DOT: {:?}", why),
    }
}

fn do_filter(
    graph: &MutationGraph,
    predecessors: Option<Sha1String>,
    leaves: bool,
) -> Result<MutationGraph, MutationGraphError> {
    let base_nodes = if let Some(node) = predecessors {
        HashSet::from_iter(graph.self_and_its_predecessors_of(&node)?.iter().cloned())
    } else {
        HashSet::new()
    };

    let filtered_nodes = if leaves {
        let mut filtered_nodes = HashSet::new();
        let leaves = graph.leaves();
        for node in base_nodes {
            filtered_nodes.insert(node);
            if let Some(children) = graph.children_of(node) {
                // ここがバグっぽい
                log::trace!("node = {} -> children = {:?}", node, children);
                for child in children.iter() {
                    if leaves.contains(child) {
                        log::trace!("leaf = {}", child);
                        filtered_nodes.insert(child);
                    }
                }
            }
        }
        filtered_nodes
    } else {
        base_nodes
    };

    let mut filtered_graph = MutationGraph::new();
    for node in filtered_nodes.iter() {
        match graph.get_node(node) {
            Some(node) => filtered_graph.add_node(node),
            None => return Err(MutationGraphError::NodeNotExists(node.to_string())),
        }
    }
    for edge in graph.edges() {
        if filtered_nodes.contains(&edge.parent) && filtered_nodes.contains(&edge.child) {
            filtered_graph.add_edge(edge)
        }
    }

    Ok(filtered_graph)
}
