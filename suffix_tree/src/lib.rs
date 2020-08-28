// #![allow(dead_code, unused_mut, unused_assignments, unused_variables)] // While writing

// Some inspiration taken from https://github.com/BurntSushi/suffix
use std::cell::Cell;
use std::collections::BTreeMap;
use std::rc::Rc;

type NodeId = usize;

// Enum used for the "characters" in a label
// where the separator will be a unique one
// in *any* string, and can thus be used to
// ensure that the Suffix Tree is finished in
// a single pass (satisfying the online condition)
// Todo: Smthing implicit/explicit Suffix Tree?
enum Test {
    Byte(u8),
    Sep, // Unique separator
}

#[derive(Debug)]
pub struct SuffixTree {
    raw_string: String,
    nodes: Vec<Node>,
    // root: Box<Node<'s>>,
}

#[derive(Debug)]
pub struct Node {
    id: NodeId,
    parent: Option<NodeId>,
    // Todo: Verify BTreeMap performance, since the
    // implementation is O(n) in Rust, but it utilizes
    // good cache-lookup for speeding up
    // Initial guess is that it would be pretty bad in
    // nodes with a large amount of children, but VERY
    // fast in a node with a limited amount of children
    children: BTreeMap<u8, NodeId>,
    suffix_link: Option<NodeId>,

    start: usize,
    end: Rc<Cell<usize>>,
}

impl SuffixTree {
    pub fn new(s: &str) -> SuffixTree {
        init_suffix_tree(s)
    }

    pub fn string(&self) -> &str {
        &self.raw_string
    }

    // Gets the byte label going into the node
    pub fn label_of_node(&self, node: &Node) -> &[u8] {
        // Need to evaluate tactic to handle this...
        // Options: panic (as is), Result<>, or format
        &self.raw_string.as_bytes()[node.start as usize..node.end.get()]
    }

    pub fn label_of_node_formatted(&self, node: &Node) -> String {
        let bytes = self.label_of_node(node);
        match String::from_utf8(bytes.to_vec()) {
            Ok(s) => s,
            Err(_) => format!("{:?}", bytes),
        }
    }

    // pub fn longest_substring(&self, bytes: &[u8]) -> (usize, usize) {
    //     find_longest_substring(self, bytes)
    // }

    // Other possible methods
    // pub fn contains(&self, s: &str) -> bool
    pub fn root(&self) -> &Node {
        &self.nodes[0]
    }

    pub fn node(&self, id: NodeId) -> &Node {
        &self.nodes[id]
    }
}

impl Node {
    pub fn new(
        id: NodeId,
        parent: Option<NodeId>,
        start: usize,
        global_end: &Rc<Cell<usize>>,
    ) -> Node {
        Node {
            id,
            parent,
            children: BTreeMap::new(),
            suffix_link: None,
            start,
            end: Rc::clone(global_end),
        }
    }

    pub fn has_child(&self, b: &u8) -> bool {
        self.children.contains_key(b)
    }

    pub fn child(&self, b: &u8) -> Option<&NodeId> {
        self.children.get(b)
    }

    pub fn children(&self) -> &BTreeMap<u8, NodeId> {
        &self.children
    }

    // pub fn root(&self) -> &Node {
    //     let mut cur = self;
    //     loop {
    //         match &cur.parent {
    //             None => break,
    //             Some(x) => cur = &x,
    //         }
    //     }

    //     cur
    // }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.end.get()
    }
}

// Todo: Issues
// - Unused character used to finish the
//   Suffix Tree, usually $ in literature.
fn init_suffix_tree(s: &str) -> SuffixTree {
    // Todo: Verify this can start at 0, examples start it at -1
    let global_end = Rc::new(Cell::new(0));
    let root = Node::new(0, None, 0, &global_end);
    let mut nodes = vec![root];
    let mut suffix_tree = SuffixTree {
        raw_string: String::from(s),
        nodes: vec![],
    };

    let string_bytes = s.as_bytes();

    // Various control variables
    let mut last_new_node: Option<NodeId>;
    let mut active_node: NodeId = 0;
    // 'active_edge' is the index of the actual byte
    // in 'string_bytes'
    // string_bytes[active_edge]
    // would give the current byte
    let mut active_edge: usize = 0;
    let mut active_length: usize = 0;
    let mut remaining_suffix_count: usize = 0;

    for (i, b) in s.bytes().enumerate() {
        // Update global_end and increment remaining suffix
        global_end.set(global_end.get() + 1);
        remaining_suffix_count += 1;

        // Clear last new node
        last_new_node = None;

        // Need to create these many suffixes, or short-circuit
        // them for next byte.
        while remaining_suffix_count > 0 {
            // If active length is 0 then it's always from root
            if active_length == 0 {
                // Check if next byte already exists from root
                if let Some(node) = nodes[active_node].child(&b) {
                    // It exists, so just update edge and length
                    // Rule 3 extension
                    active_edge = nodes[*node].start;
                    active_length += 1;
                    break;
                } else {
                    // It does not exist yet, so we create it
                    // Rule 2 extension
                    let new_node =
                        Node::new(nodes.len(), Some(nodes[active_node].id), i, &global_end);
                    // active_node is root in this case
                    nodes[active_node].children.insert(b, new_node.id);
                    nodes.push(new_node);

                    // And update remaining_suffix_count since
                    // a node was created
                    remaining_suffix_count -= 1;
                }
            } else {
                // Active length not 0, so traversing somewhere at the moment
                // Check if next character is the same
                let next_byte: u8;

                let n = *nodes[active_node]
                    .child(&string_bytes[active_edge])
                    .unwrap();
                let node = &nodes[n];
                let mut label = suffix_tree.label_of_node(node);
                // If the active_length is greater than the label,
                // then the next character might exist in a child
                // of the node
                if active_length > label.len() {
                    if let Some(child_node) = node.child(&string_bytes[active_edge]) {
                        label = suffix_tree.label_of_node(&nodes[*child_node]);
                        // Because we did an internal jump, the active point
                        // has to be updated
                        active_node = *child_node;
                        active_edge = nodes[*child_node].start;
                        active_length = 0;
                    }
                }
                next_byte = label[active_length];
                if next_byte == b {
                    // Next byte match, so continue along the edge (skip to next)
                    // Also known as a Rule 3 extension
                    active_length += 1;
                    break;
                } else {
                    // Does not match, have to create a new internal node
                    // Rule 2 extension
                    // - Creates new internal node, splitting up the path
                    // - Decrements active_length by 1
                    // - Increments active_edge by 1

                    let nodes_len = nodes.len();
                    let root_id = nodes[0].id;

                    // Have to override n and node here in order to
                    // not mess with rust's borrowing rules...
                    // Todo: There might be a better way?
                    let n = *nodes[active_node]
                        .child(&string_bytes[active_edge])
                        .unwrap();
                    let node = &mut nodes[n];
                    let new_node = Node::new(nodes_len, Some(node.id), active_length, &global_end);
                    let new_node2 = Node::new(nodes_len + 1, Some(node.id), i, &global_end);
                    let node_to_update_id = node.id;
                    node.end = Rc::new(Cell::new(active_length - 1));
                    node.children
                        .insert(string_bytes[new_node.start], new_node.id);
                    node.children
                        .insert(string_bytes[new_node2.start], new_node2.id);
                    // Set suffix links. The new node is set to root, and
                    // last_new_node (if exists) is set to the new node.
                    node.suffix_link = Some(root_id);
                    if let Some(last_new_node_id) = last_new_node {
                        nodes[last_new_node_id].suffix_link = Some(node_to_update_id);
                    }
                    nodes.push(new_node);
                    nodes.push(new_node2);

                    // Rule 2 extension rules
                    remaining_suffix_count -= 1;
                    active_length -= 1;
                    active_edge += 1;
                    // active_node as well?
                    last_new_node = Some(node_to_update_id);
                }
            }
        }
    }

    suffix_tree.nodes = nodes;
    suffix_tree
}

// fn find_longest_substring(tree: &SuffixTree, bytes: &[u8]) -> (usize, usize) {
//     let mut current_node = tree.root();
//     let mut position_in_bytes = 0;

//     // Edge case for the beginning, is used to find the start position
//     // Todo: This might panic
//     current_node = current_node.children().get(&bytes[0]).unwrap();
//     let start = current_node.start() as usize;

//     // First label might be longer than 1 byte, so check along it
//     for b in tree.label_of_node(&current_node) {
//         if b == &bytes[position_in_bytes] {
//             position_in_bytes += 1;
//             if position_in_bytes >= bytes.len() {
//                 return (start, start + position_in_bytes);
//             }
//         } else {
//             // If it does not contain the entire label of
//             // the first child, then the indexing is rather simple
//             return (start, start + position_in_bytes);
//         }
//     }

//     // If we got to here, then there exists a first child whose whole
//     // label was contained in bytes. Not we check subsequent children,
//     // until either the longest substring is found, or there are no
//     // more 'bytes'.
//     // Todo: verify termination
//     loop {
//         // Exit if current_node has no children
//         // or if the next byte is not a key for a child
//         let children = current_node.children();
//         if children.is_empty() || children.contains_key(&bytes[position_in_bytes as usize]) {
//             return (start, start + position_in_bytes);
//         }

//         // Check along the label
//         let child = children.get(&bytes[position_in_bytes as usize]).unwrap();
//         for (index, b) in tree.label_of_node(&child).iter().enumerate() {
//             if b == &bytes[position_in_bytes as usize] {
//                 position_in_bytes += 1;
//                 if position_in_bytes >= bytes.len() {
//                     return (start, (current_node.start() as usize) + index - 1);
//                 }
//             } else {
//                 return (start, (current_node.start() as usize) + index - 1);
//             }
//         }

//         // All of the label was there, so update current_node
//         current_node = child;
//     }

//     // (start, start + position_in_bytes as u32)
// }

#[cfg(test)]
mod tests {
    // use quickcheck::quickcheck;
    use super::*;

    #[test]
    fn basic() {
        let st = SuffixTree::new("banaxna");
        println!("{:?}", st.raw_string.as_bytes());
        println!("{:?}", st);
    }

    // #[test]
    // fn basic2() {
    //     SuffixTree::new("apple");
    // }

    // #[test]
    // fn basic3() {
    //     SuffixTree::new("mississippi");
    // }

    // #[test]
    // fn longest_substring1() {
    //     let tree = SuffixTree::new("banana");
    //     let bytes = "ban".as_bytes();
    //     let result = tree.longest_substring(&bytes);
    //     // let (start, end) = result;
    //     // println!("({}, {})", start, end);
    //     // println!("{:?}", tree.text().as_bytes());
    //     // println!("{:?}", &tree.text().as_bytes()[start..end]);
    //     assert_eq!((0, 3), result);
    // }

    // #[test]
    // fn longest_substring2() {
    //     let tree = SuffixTree::new("banana");
    //     let bytes = "anana".as_bytes();
    //     let result = tree.longest_substring(&bytes);
    //     assert_eq!((1, 6), result);
    // }

    // #[test]
    // fn qc_n_leaves() {
    //     fn prop(s: String) -> bool {
    //         SuffixTree::new(&*s).root.leaves().count() == s.len()
    //     }
    //     quickcheck(prop as fn(String) -> bool);
    // }

    // #[test]
    // fn qc_internals_have_at_least_two_children() {
    //     fn prop(s: String) -> bool {
    //         let st = SuffixTree::new(&*s);
    //         for node in st.root.preorder() {
    //             if !node.has_terminals() && node.children.len() < 2 {
    //                 return false;
    //             }
    //         }
    //         true
    //     }
    //     quickcheck(prop as fn(String) -> bool);
    // }

    // #[test]
    // fn qc_tree_enumerates_suffixes() {
    //     fn prop(s: String) -> bool {
    //         // This is pretty much relying on `SuffixTable::new_naive` to
    //         // produce the correct suffixes. But the nice thing about the naive
    //         // algorithm is that it's stupidly simple.
    //         let sa = SuffixTable::new(&*s);
    //         let st = SuffixTree::from_suffix_table(&sa);
    //         for (i, sufi) in st.root.suffix_indices().enumerate() {
    //             if &st.text.as_bytes()[sufi as usize..] != sa.suffix_bytes(i) {
    //                 return false;
    //             }
    //         }
    //         true
    //     }
    //     quickcheck(prop as fn(String) -> bool);
    // }
}
