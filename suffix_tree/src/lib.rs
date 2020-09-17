// Some inspiration for structure taken from the "Suffix Tree" part of https://github.com/BurntSushi/suffix
// Link at time of writing: https://github.com/BurntSushi/suffix/blob/5dac1121d901f13b48f41847c9ccb8fd18070199/suffix_tree/src/lib.rs

// However for the actual creation of the Suffix Tree, inspiration was taken from
// https://www.geeksforgeeks.org/ukkonens-suffix-tree-construction-part-6/
// Archived via web.archive.org on 14/09/2020

use std::cell::Cell;
use std::fmt;
use std::iter;
use std::rc::Rc;

// Declaring the label_data and node modules without explicitly having a
// mod.rs file in types/mod.rs or a types.rs in src, essentially saving
// a file that just declares the modules.
mod types {
    pub mod label_data;
    pub mod node;
}
use types::label_data::LabelData;
use types::node::{Node, NodeId};

pub struct SuffixTree {
    raw_string: String,
    nodes: Vec<Node>,
    string: Vec<LabelData>,
}

impl SuffixTree {
    pub fn new<T: AsRef<str>>(s: T) -> Self {
        internal_to_suffix_tree(s)
    }

    pub fn string(&self) -> &str {
        &self.raw_string
    }

    // Gets the byte label going into the node
    pub fn label_of_node(&self, node: &Node) -> &[LabelData] {
        &self.string[node.start..node.end.get()]
    }

    pub fn label_of_node_formatted(&self, node: &Node) -> String {
        // Turns the LabelData into a readable format
        // i.t. the LabelData::Sep is made into the
        //  &'static str SEP value and because it is
        // a vector of bytes, it needs to be flattened
        // and then collected back into a single vector.
        let label_data = self
            .label_of_node(node)
            .into_iter()
            .map(|l| l.as_readable())
            .flatten()
            .collect::<Vec<_>>();

        // We need to clone the label_data because
        // String::from_utf8 takes ownership of the
        // string, but we need it in case it fails
        // to create it so we can format it ourselves
        match String::from_utf8(label_data.clone()) {
            Ok(s) => s,
            Err(_) => format!("{:?}", label_data),
        }
    }

    pub fn contains_suffix(&self, suffix: &[u8]) -> bool {
        internal_contains_suffix(self, suffix)
    }

    // pub fn contains_substring(&self, substr: &[u8]) -> bool {
    //     internal_contains_substring(self, substr)
    // }

    pub fn longest_substring(&self, substr: &[u8]) -> Option<(usize, usize)> {
        internal_longest_substring(self, substr)
    }

    pub fn root(&self) -> &Node {
        &self.nodes[0]
    }

    pub fn node(&self, id: NodeId) -> &Node {
        &self.nodes[id]
    }
}

impl fmt::Debug for SuffixTree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fn fmt(f: &mut fmt::Formatter, st: &SuffixTree, node: &Node, depth: usize) -> fmt::Result {
            let indent: String = iter::repeat(' ').take(depth * 2).collect();
            if node.is_root() {
                writeln!(f, "ROOT")?;
            } else {
                writeln!(
                    f,
                    "{}{:?} ({:?}: {}, {:?}) --- {:?}",
                    indent,
                    &st.label_of_node_formatted(node),
                    node.suffix_index.unwrap(),
                    node.start,
                    node.end.get(),
                    &st.raw_string[node.suffix_range()]
                )?;
            }
            for child in node.children().values() {
                fmt(f, st, &st.nodes[*child], depth + 1)?;
            }
            Ok(())
        }
        writeln!(f, "\n-----------------------------------------")?;
        writeln!(f, "SUFFIX TREE")?;
        writeln!(f, "raw string: {}", self.raw_string)?;
        fmt(f, self, self.root(), 0)?;
        writeln!(f, "-----------------------------------------")
    }
}

fn internal_to_suffix_tree<T: AsRef<str>>(s: T) -> SuffixTree {
    // Mutable global end, only possible via
    // the Cell container.
    let global_end = Rc::new(Cell::new(0));
    // Root always has id 0, no parent, start is 0 and
    // a reference to the global end
    let root = Node::new(0, None, None, 0, &global_end);
    // The nodes vector contains all of the nodes,
    // where they just have the Id's of the referenced
    // nodes in accordance to this list.
    let mut nodes = vec![root];

    // Transforms the input string into a list of
    // bytes, wrapped into the LabelData enum
    // and lastly appends the separator at the
    // end of this list. This ensures a unique
    // last byte to finish up the suffix tree.
    let mut bytes_and_sep = s
        .as_ref()
        .as_bytes()
        .into_iter()
        .map(|&b| LabelData::new(b))
        .collect::<Vec<_>>();
    bytes_and_sep.push(LabelData::Sep);

    let mut suffix_tree = SuffixTree {
        raw_string: String::from(s.as_ref()),
        nodes: vec![],
        string: vec![],
    };

    // Various control variables
    let mut last_new_node: Option<NodeId>;
    // Root node is always the first in the list
    let root_id = 0;
    let mut active_node: NodeId = root_id;
    // 'active_edge' is the index of the actual LabelData
    // in 'bytes_and_sep'. bytes_and_sep[active_edge]
    // would give the current LabelData
    let mut active_edge: usize = 0;
    let mut active_length: usize = 0;
    let mut remaining_suffix_count: usize = 0;

    // Returns Option<(NodeId, Length, Edge)>
    fn walk_down(
        nodes: &Vec<Node>,
        node_id: NodeId,
        act_l: usize,
        act_e: usize,
    ) -> Option<(NodeId, usize, usize)> {
        let label_len = nodes[node_id].length();
        if act_l >= label_len {
            let e = act_e + label_len;
            let l = act_l - label_len;
            let n = node_id;
            return Some((n, l, e));
        }
        None
    }

    // Build up the entire node tree with all the relations
    for (i, &b) in bytes_and_sep.iter().enumerate() {
        // Update global_end and increment remaining suffix
        // Extension rule 1 for global_end
        global_end.set(global_end.get() + 1);
        remaining_suffix_count += 1;

        // Clear last new node
        last_new_node = None;

        // Need to create these many suffixes, or short-circuit
        // them for next byte.
        while remaining_suffix_count > 0 {
            if active_length == 0 {
                active_edge = i;
            }

            if !nodes[active_node].has_child(&bytes_and_sep[active_edge]) {
                // Rule 2 extension
                let new_node = Node::new(
                    nodes.len(),
                    Some(active_node),
                    Some(root_id),
                    i,
                    &global_end,
                );
                let node = &mut nodes[active_node];
                node.children
                    .insert(bytes_and_sep[active_edge], new_node.id);
                nodes.push(new_node);

                // If a node was created in the last iteration,
                // then we need to set the suffix link of that
                // to the current active node.
                if let Some(last_new_node_id) = last_new_node {
                    nodes[last_new_node_id].suffix_link = Some(active_node);
                    last_new_node = None;
                }
            } else {
                let next = *nodes[active_node]
                    .child(&bytes_and_sep[active_edge])
                    .unwrap();
                if let Some((n, l, e)) = walk_down(&nodes, next, active_length, active_edge) {
                    active_node = n;
                    active_length = l;
                    active_edge = e;
                    continue; // Need to continue walkdown from next node
                }

                // Extension rule 3
                if bytes_and_sep[nodes[next].start + active_length] == b {
                    // Check if suffix link needs to be set
                    // Apparently Rust does not yet allow "if let X &&"
                    // expressions, so will have to live with this nested if
                    if let Some(last_new_node_id) = last_new_node {
                        if active_node != root_id {
                            nodes[last_new_node_id].suffix_link = Some(active_node);
                        }
                    }

                    // Increment active_length and break, show stopper
                    active_length += 1;
                    break;
                }

                // Extension rule 2
                // New character is currently not in the label
                // so will have to create a new internal node,
                // and a new leaf node.
                let curr_node_start = nodes[next].start;
                let split_end = Rc::new(Cell::new(curr_node_start + active_length));
                let mut split_node = Node::new(
                    nodes.len(),
                    nodes[next].parent,
                    Some(root_id),
                    curr_node_start,
                    &split_end,
                );
                nodes[active_node]
                    .children
                    .insert(bytes_and_sep[active_edge], split_node.id);
                let new_leaf = Node::new(
                    nodes.len() + 1,
                    Some(split_node.id),
                    Some(root_id),
                    i,
                    &global_end,
                );
                split_node.children.insert(bytes_and_sep[i], new_leaf.id);
                nodes[next].start += active_length;
                nodes[next].parent = Some(split_node.id);

                split_node
                    .children
                    .insert(bytes_and_sep[nodes[next].start], next);

                let split_node_id = split_node.id;
                nodes.push(split_node);
                nodes.push(new_leaf);

                if let Some(last_new_node_id) = last_new_node {
                    nodes[last_new_node_id].suffix_link = Some(split_node_id);
                }
                last_new_node = Some(split_node_id);
            }

            remaining_suffix_count -= 1;
            if active_node == root_id && active_length > 0 {
                active_length -= 1;
                active_edge = i - remaining_suffix_count + 1;
            } else if active_node != root_id {
                active_node = nodes[active_node].suffix_link.unwrap();
            }
        }
    }

    // Now to actually be able to find the suffix
    // index for a given node, we need to run a
    // traversal on the tree, and the index is then
    // found by `s.len() - label_height`.
    let mut stack = vec![(root_id, 0)];
    while let Some((node_id, label_height)) = stack.pop() {
        let new_height;
        if node_id != root_id {
            nodes[node_id].suffix_index = Some(nodes[node_id].start - label_height);
            new_height = label_height + nodes[node_id].length();
        } else {
            new_height = label_height;
        }
        for n in nodes[node_id].children().values() {
            stack.push((*n, new_height));
        }
    }

    suffix_tree.nodes = nodes;
    suffix_tree.string = bytes_and_sep;
    suffix_tree
}

fn internal_contains_suffix(st: &SuffixTree, suffix: &[u8]) -> bool {
    // While the empty string is strictly a
    // suffix, I'm not sure if it makes sense
    // in practice, so for now just discard it
    if suffix.len() == 0 {
        return false;
    }

    // Always starts from root
    let nodes = &st.nodes;
    let mut cur_node = st.root();
    let mut i = 0;
    let mut suffix_label_data: Vec<_> = suffix.iter().map(|&b| LabelData::new(b)).collect();
    suffix_label_data.push(LabelData::Sep);
    while i < suffix_label_data.len() {
        if let Some(new_node_id) = cur_node.child(&suffix_label_data[i]) {
            // Check if label is longer than 1
            let label = st.label_of_node(&nodes[*new_node_id]);
            if label.len() > 1 {
                // If it is, we also need to check the characters
                // in suffix, to make sure that they all match.
                for j in 1..label.len() {
                    if i + j >= suffix_label_data.len() || suffix_label_data[i + j] != label[j] {
                        return false;
                    }
                }

                // If it succeeded, update i by label.len
                i += label.len() - 1;
            }
            cur_node = &nodes[*new_node_id];
        } else {
            return false;
        }
        i += 1;
    }
    cur_node.is_leaf()
}

// Returns the starting index of the substring, and the ending index (not inclusive)
// if one exists, otherwise returns None
fn internal_longest_substring(st: &SuffixTree, bytes: &[u8]) -> Option<(usize, usize)> {
    if bytes.len() == 0 {
        // Todo: Panic or return None?
        panic!("No bytes left to find substring on");
        // return None;
    }

    // Todo: Need to find the last node,
    // then can just return the node.suffix_range()
    // start and end as a tuple.
    let nodes = &st.nodes;
    let mut cur_node = st.root();
    let mut i = 0;
    while i < bytes.len() {
        if let Some(next_node_id) = cur_node.child(&LabelData::new(bytes[i])) {
            let label = st.label_of_node(&nodes[*next_node_id]);
            if label.len() > 1 {
                // If it is, we also need to check the characters
                // in suffix, to make sure that they all match.
                for j in 1..label.len() {
                    // If it runs out of bytes to check, or a byte does not
                    // match, then the longest substring is found
                    if i + j >= bytes.len() || bytes[i + j] != label[j] {
                        let start = nodes[*next_node_id].suffix_index.unwrap();
                        return Some((start, start + i + j));
                    }
                }

                // If it succeeded, update i by label.len
                i += label.len() - 1;
            }
            cur_node = &nodes[*next_node_id];
        } else {
            // Next byte does not fit, break loop
            break;
        }
        i += 1;
    }

    // If the current node is still root, then
    // no substring exists
    if cur_node.is_root() {
        return None;
    }

    // If this point is ever reached, it is because
    // it did not break early (in the middle of a label)
    // so we can return the suffix_range of the current
    // node.
    let range = cur_node.suffix_range();
    Some((range.start, range.end))
}

#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::quickcheck;

    #[test]
    fn basic() {
        SuffixTree::new("banana");
    }

    #[test]
    fn basic2() {
        SuffixTree::new("xyzaxyzbcyzd");
    }

    #[test]
    fn basic3() {
        SuffixTree::new("mississippi");
    }

    #[test]
    fn utf8_japanese() {
        SuffixTree::new("ゴム製のアヒル");
    }

    #[test]
    fn utf8_chinese() {
        SuffixTree::new("橡皮鸭");
    }

    #[test]
    fn longest_substring1() {
        let tree = SuffixTree::new("banana");
        let string = "ban";
        let result = tree.longest_substring(string.as_bytes()).unwrap();
        assert_eq!((0, 3), result);
    }

    #[test]
    fn longest_substring2() {
        let tree = SuffixTree::new("banana");
        let string = "anana";
        let result = tree.longest_substring(string.as_bytes()).unwrap();
        assert_eq!((1, 6), result);
    }

    #[test]
    fn longest_substring3() {
        let tree = SuffixTree::new("mississippi");
        let string = "issi";
        let result = tree.longest_substring(string.as_bytes()).unwrap();
        assert_eq!((1, 5), result);
    }

    #[test]
    fn longest_substring4() {
        let tree = SuffixTree::new("mississippi");
        let string = "issip";
        let result = tree.longest_substring(string.as_bytes()).unwrap();
        assert_eq!((4, 9), result);
    }

    #[test]
    fn longest_substring5() {
        let tree = SuffixTree::new("banana");
        let string = "anab";
        let result = tree.longest_substring(string.as_bytes()).unwrap();
        assert_eq!((1, 4), result);
    }

    #[test]
    fn longest_substring_none() {
        let tree = SuffixTree::new("banana");
        let string = "xqr";
        let result = tree.longest_substring(string.as_bytes());
        assert_eq!(None, result);
    }

    #[test]
    fn does_not_contain_empty_string_as_suffix() {
        let st = SuffixTree::new("banana");
        let empty = [];
        assert!(!st.contains_suffix(&empty));
    }

    // There are str.len() + 1 leaves since the
    // separator is also added as a leaf from the root.
    #[quickcheck]
    fn amount_of_leaves_is_len_plus_one(s: String) -> bool {
        SuffixTree::new(&s)
            .nodes
            .iter()
            .filter(|&n| n.is_leaf())
            .count()
            == s.len() + 1
    }

    #[quickcheck]
    fn contains_all_suffixes(s: String) -> bool {
        let st = SuffixTree::new(&s);
        for i in 0..s.len() {
            if !st.contains_suffix(&s.as_bytes()[i..]) {
                return false;
            }
        }
        true
    }

    #[quickcheck]
    fn every_internal_node_has_at_least_two_children(s: String) -> bool {
        SuffixTree::new(&s)
            .nodes
            .iter()
            .all(|n| !n.is_internal_node() || n.children().values().count() >= 2)
    }
}
