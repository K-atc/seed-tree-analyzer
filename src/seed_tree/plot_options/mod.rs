use std::collections::{HashMap, HashSet};

use plot_option::PlotOption;

pub mod error;
pub mod plot_option;
pub mod result;

use crate::seed_tree::mutation_graph_edge::MutationGraphEdge;
use crate::seed_tree::node_name::NodeName;
use crate::seed_tree::plot_options::error::PlotOptionError;
use result::Result;

type Label = String;

#[derive(Debug, Eq, PartialEq, Default)]
pub struct PlotOptions {
    pub highlight_edges_from_root_to: Option<NodeName>,
    pub highlight_edge_with_blue: HashSet<MutationGraphEdge>,
    pub highlight_edge_with_red: HashSet<MutationGraphEdge>,
    pub highlight_edge_with_green: HashSet<MutationGraphEdge>,
    pub highlight_crash_input: bool,
    pub notate: HashMap<NodeName, Label>,
}

impl PlotOptions {
    pub fn none() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn from(options: &[PlotOption]) -> Result<Self> {
        Ok(Self {
            highlight_edges_from_root_to: {
                let mut nodes: HashSet<NodeName> = HashSet::new();
                for option in options.iter() {
                    match option {
                        PlotOption::HighlightEdgesFromRootTo(ref v) => {
                            nodes.insert(v.clone());
                        }
                        _ => (),
                    }
                }
                match nodes.len() {
                    0..=1 => nodes.iter().last().cloned(),
                    _ => {
                        return Err(PlotOptionError::MultiplePredecessorsNotSupported(
                            nodes.clone(),
                        ))
                    }
                }
            },
            highlight_edge_with_blue: {
                let mut edges: HashSet<MutationGraphEdge> = HashSet::new();
                for option in options.iter() {
                    match option {
                        PlotOption::HighlightEdgeWithBlue(ref v) => {
                            edges.insert(v.clone());
                        }
                        _ => (),
                    }
                }
                edges
            },
            // FIXME: Spaghetti code
            highlight_edge_with_red: {
                let mut edges: HashSet<MutationGraphEdge> = HashSet::new();
                for option in options.iter() {
                    match option {
                        PlotOption::HighlightEdgeWithRed(ref v) => {
                            edges.insert(v.clone());
                        }
                        _ => (),
                    }
                }
                edges
            },
            highlight_edge_with_green: {
                let mut edges: HashSet<MutationGraphEdge> = HashSet::new();
                for option in options.iter() {
                    match option {
                        PlotOption::HighlightEdgeWithGreen(ref v) => {
                            edges.insert(v.clone());
                        }
                        _ => (),
                    }
                }
                edges
            },
            highlight_crash_input: options.contains(&PlotOption::HighlightCrashInput),
            notate: {
                let mut notes: HashMap<NodeName, Label> = HashMap::new();
                for option in options.iter() {
                    match option {
                        PlotOption::NotateTo(ref node, ref label) => {
                            match notes.get_mut(node) {
                                Some(old_label) => {
                                    let new_label = format!("{}\n{}", old_label, label);
                                    notes.insert(node.clone(), new_label)
                                }
                                None => notes.insert(node.clone(), label.clone()),
                            };
                        }
                        _ => (),
                    }
                }
                notes
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::seed_tree::node_name::NodeName;
    use crate::seed_tree::plot_options::error::PlotOptionError;
    use crate::seed_tree::plot_options::plot_option::PlotOption;
    use crate::seed_tree::plot_options::PlotOptions;
    use std::collections::HashSet;
    use std::iter::FromIterator;

    #[test]
    fn test_plot_options_none() {
        let options = PlotOptions::from(&[]);
        assert_eq!(options, Ok(PlotOptions::none()))
    }

    #[test]
    fn test_plot_options_highlight_predecessors_of_one_node() {
        let sha1_1 = NodeName::from("1");
        let options = PlotOptions::from(&[PlotOption::HighlightEdgesFromRootTo(sha1_1.clone())]);
        assert_eq!(
            options,
            Ok(PlotOptions {
                highlight_edges_from_root_to: Some(sha1_1),
                ..Default::default()
            })
        )
    }

    #[test]
    fn test_plot_options_highlight_predecessors_of_multiple_nodes() {
        let sha1_1 = NodeName::from("1");
        let sha1_2 = NodeName::from("2");
        let options = PlotOptions::from(&[
            PlotOption::HighlightEdgesFromRootTo(sha1_1.clone()),
            PlotOption::HighlightEdgesFromRootTo(sha1_2.clone()),
        ]);
        assert_eq!(
            options,
            Err(PlotOptionError::MultiplePredecessorsNotSupported(
                HashSet::from_iter([sha1_1, sha1_2].iter().cloned())
            ))
        )
    }
}
