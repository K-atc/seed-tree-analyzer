use super::error::ParseError;
use super::result::Result;
use crate::seed_tree::mutation_graph_edge::MutationGraphEdge;
use crate::seed_tree::mutation_graph_node::MutationGraphNode;
use crate::seed_tree::util::calc_file_hash;
use crate::seed_tree::MutationGraph;

use regex::Regex;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug)]
pub struct AFLExtensions {
    pub(crate) aurora: bool,
    pub(crate) crash_inputs_dir: Option<PathBuf>,
}

impl AFLExtensions {
    pub fn none(&self) -> bool {
        !self.aurora
    }

    pub fn aurora(&self) -> bool {
        self.aurora
    }
}

pub fn parse_afl_input_directories<T: AsRef<Path>>(
    directories: HashSet<T>,
    extensions: &AFLExtensions,
) -> Result<MutationGraph> {
    let mut res = MutationGraph::new();
    for directory in directories {
        parse_afl_input_directory(directory, &mut res, extensions)?;
    }
    Ok(res)
}

fn parse_afl_input_directory<T: AsRef<Path>>(
    directory: T,
    graph: &mut MutationGraph,
    extensions: &AFLExtensions,
) -> Result<()> {
    if directory.as_ref().is_file() {
        return Err(ParseError::UnexpectedFilePath(
            directory.as_ref().to_path_buf(),
        ));
    }

    visit_directory(directory.as_ref().to_path_buf(), graph, extensions)?;

    Ok(())
}

fn visit_directory(
    directory: PathBuf,
    graph: &mut MutationGraph,
    extensions: &AFLExtensions,
) -> Result<()> {
    log::trace!("Scanning directory {:?}", directory);

    let one_line_info = Regex::new(if extensions.aurora() {
        "^id:(\\d+)(?:,sig:\\d+)?,(?:src|orig):([^:]+)(?:,op:([^_]+)(?:_(\\S+))?)?$"
        //   ~~~~~~1                           ~~~~~~~2      ~~~~~~~3 ~~~~~~4
    } else {
        "^id:(\\d+)(?:,sig:\\d+)?(?:,time:\\d+)?(?:,execs:\\d+)?,(?:src|orig):([^:]+)(?:,time:\\d+)?(?:,execs:\\d+)?(?:,op:(\\S+))?$"
        //   ~~~~~~1                                                          ~~~~~~~2                                     ~~~~~~3
    })?;

    for entry in directory.read_dir()? {
        let file_path = entry?.path();

        // Recursively iterate directory
        if file_path.is_dir() {
            if file_path.file_name() == Some(OsStr::new(".state")) {
                log::warn!("Skipped directory {:?}", file_path);
            } else {
                visit_directory(file_path, graph, extensions)?;
            }
            continue;
        }

        // log::trace!("parsing file name: {}", file_name);
        let file_name = match file_path.file_name() {
            Some(file_name) => file_name.to_str().ok_or(ParseError::StringEncoding)?,
            None => return Err(ParseError::UnexpectedFilePath(file_path)),
        };

        let is_crash_input_node = match extensions.crash_inputs_dir {
            Some(ref crash_input_dir) => &directory == crash_input_dir,
            None => directory.ends_with("crashes"),
        };

        match one_line_info.captures(file_name) {
            Some(captures) => {
                let id = match captures.get(1) {
                    Some(id) => {
                        if extensions.aurora() {
                            match captures.get(4) {
                                Some(non_crash_id) => {
                                    format!("nc-{}", non_crash_id.as_str())
                                }
                                None => id.as_str().to_string(),
                            }
                        } else {
                            if is_crash_input_node {
                                format!("crash-{}", id.as_str())
                            } else {
                                id.as_str().to_string()
                            }
                        }
                    }
                    None => {
                        return Err(ParseError::SyntaxError(
                            "'id' does not exists",
                            file_name.to_string(),
                        ))
                    }
                };
                graph.add_node(&MutationGraphNode::new_with_metadata(
                    &id.to_string(),
                    is_crash_input_node,
                    file_path.as_path(),
                    &calc_file_hash(file_path.as_path())?,
                ));

                let src_list = match captures.get(2) {
                    Some(src_list) => src_list.as_str().split("+"),
                    None => {
                        return Err(ParseError::SyntaxError(
                            "'src' does not exists",
                            file_name.to_string(),
                        ))
                    }
                };
                let op = match captures.get(3) {
                    Some(op) => op.as_str(),
                    None => "origin",
                };

                for src in src_list {
                    graph.add_edge(&MutationGraphEdge {
                        parent: src.to_string(),
                        child: id.to_string(),
                        label: op.to_string(),
                    });
                    break; // Ignore splice source input
                }
            }
            None => {
                if file_name.starts_with("id:") {
                    log::warn!(
                        "file \"{}\" does not have AFL's input file name format",
                        file_name
                    )
                } else {
                    if file_name == "README.txt" {
                        log::info!("README file \"{}\" found. Skip", file_name)
                    } else {
                        graph.add_node(&MutationGraphNode::new_with_metadata(
                            &file_name.to_string(),
                            false,
                            &file_path,
                            &calc_file_hash(file_path.as_path())?,
                        ))
                    }
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use crate::seed_tree::mutation_graph_node::MutationGraphNode;
    use crate::seed_tree::node_name::NodeName;
    use crate::seed_tree::parser::afl::{parse_afl_input_directory, AFLExtensions};
    use crate::seed_tree::MutationGraph;
    use std::collections::HashSet;
    use std::iter::FromIterator;
    use std::path::Path;

    macro_rules! node {
        ( $x:expr ) => {
            &NodeName::from($x)
        };
    }

    #[test]
    fn test_aflplusplus_seed_tree() {
        let seed_dir = Path::new("test/sample/seed-tree/aflplusplus-4.05c/");

        let mut graph = MutationGraph::new();
        assert!(parse_afl_input_directory(
            seed_dir,
            &mut graph,
            &AFLExtensions {
                aurora: false,
                crash_inputs_dir: Some(seed_dir.join("crashes/"))
            }
        )
        .is_ok());

        // println!("{:#?}", graph);

        match graph.get_node(&String::from("000000")) {
            Some(node) => {
                let expected = MutationGraphNode {
                    name: String::from("000000"),
                    crashed: false,
                    file: seed_dir.join("queue/id:000000,time:0,execs:0,orig:hello.attach-123.pdf"),
                    hash: String::from("da39a3ee5e6b4b0d3255bfef95601890afd80709"),
                };
                assert_eq!(node, &expected);
                assert_eq!(node.crashed, expected.crashed);
                assert_eq!(node.file, expected.file);
                assert_eq!(node.hash, expected.hash);
            }
            None => unreachable!(),
        }

        match graph.get_node(&String::from("crash-000002")) {
            Some(node) => {
                let expected = MutationGraphNode { name: String::from("crash-000002"), crashed: true, file: seed_dir.join("crashes/id:000002,sig:06,src:000000,time:8024,execs:2409,op:colorization,pos:0"), hash: String::from("7e240de74fb1ed08fa08d38063f6a6a91462a815") };
                assert_eq!(node, &expected);
                assert_eq!(node.crashed, expected.crashed);
                assert_eq!(node.file, expected.file);
                assert_eq!(node.hash, expected.hash);
            }
            None => unreachable!(),
        }
    }

    #[test]
    fn test_aurora_seed_tree() {
        let mut graph = MutationGraph::new();
        assert!(parse_afl_input_directory(
            "test/sample/seed-tree/afl-aurora-crash-exploration/",
            &mut graph,
            &AFLExtensions {
                aurora: true,
                crash_inputs_dir: Some(
                    Path::new("test/sample/seed-tree/afl-aurora-crash-exploration/crashes/")
                        .to_path_buf()
                )
            }
        )
        .is_ok());

        assert_eq!(
            graph.roots(),
            HashSet::from_iter([node!("crash-40fc056ab481fe4adb78715ea20a0fa486c81ec9")])
        );
        assert_eq!(
            graph.leaves(),
            HashSet::from_iter([
                node!("000004"),
                node!("000005"),
                node!("000006"),
                node!("000008"),
                node!("000009"),
                node!("000010"),
                node!("000011"),
                node!("000012"),
                node!("000013"),
                node!("000014"),
                node!("nc-143"),
                node!("nc-228"),
                node!("nc-298"),
                node!("nc-348"),
            ])
        );
    }
}
