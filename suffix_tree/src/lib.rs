// #![allow(dead_code, unused_mut, unused_assignments, unused_variables)] // While writing

// Some inspiration taken from https://github.com/BurntSushi/suffix
use std::cell::Cell;
use std::collections::BTreeMap;
use std::rc::Rc;

type NodeId = usize;

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
    path_len: usize,
}

impl SuffixTree {
    pub fn new(s: &str) -> SuffixTree {
        let c = find_unique_separator(s);
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
    pub fn new(id: NodeId, global_end: &Rc<Cell<usize>>) -> Node {
        Node {
            id,
            parent: None,
            children: BTreeMap::new(),
            suffix_link: None,
            start: 0,
            end: Rc::clone(global_end),
            path_len: 0,
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

fn find_unique_separator(s: &str) -> char {
    // Todo, this logic...
    '$'
}

// Todo: Issues
// - Unused character used to finish the
//   Suffix Tree, usually $ in literature.
fn init_suffix_tree(s: &str) -> SuffixTree {
    let global_end = Rc::new(Cell::new(0));
    let mut id = 0;
    let mut root = Node::new(id, &global_end);
    id += 1;
    let mut nodes = vec![root];

    {
        // Various control variables
        let mut last_new_node: Option<&Node> = None;
        let mut active_node: NodeId = 0;
        let mut active_edge = 0;
        let mut active_length = 0;
        // let mut active_point: Option<(Box<Node>, char, usize)> = None;
        let mut remaining_suffix_count = 0;

        for (i, b) in s.bytes().enumerate() {
            // Update leaf_end and increment remaining suffix
            global_end.set(i); // Should increment by 1
            remaining_suffix_count += 1;

            // Clear last new node
            last_new_node = None;

            // Need to create these many suffixes, or short-circuit
            // them for next byte.
            while remaining_suffix_count > 0 {
                // If active length is 0 then it's always from root
                if active_length == 0 {
                    // Check if next byte already exists from root
                    if nodes[active_node].has_child(&b) {
                        // It exists, so just update edge and length
                        active_edge = b;
                        active_length += 1;
                        break;
                    } else {
                        // It does not exist yet, so we create it
                        let mut new_node = Node::new(id, &global_end);
                        id += 1;
                        new_node.parent = Some(nodes[active_node].id);
                        new_node.start = i;
                        nodes[active_node].children.insert(b, new_node.id); // active_node is root
                        nodes.push(new_node);

                        // And update remaining_suffix_count since
                        // a node was created
                        remaining_suffix_count -= 1;
                    }
                } else {
                    // Active length not 0, so traversing somewhere at the moment
                    // Check if next character is the same
                    nodes[active_node].child(active_edge)
                }
            }

            // Check if next char already exists from current active node

            //
        }

        // for i in 0..size {
        //   // Extension rule 1, all nodes' end are
        //   // updated since they contain a reference
        //   // to leaf_end
        //   leaf_end = i;

        //   remaining_suffix_count++;

        //   // New phase, no new nodes are to be considered
        //   last_new_node = None;

        //   while remaining_suffix_count > 0 {

        //   }
        // }
    }

    // Fix dangling references to end in O(n) time
    // let mut nodes = vec![&mut root];
    // while let Some(n) = nodes.pop() {
    //     match n.tmp_end {
    //         Some(i) => {
    //             n.end = *i.borrow();
    //             n.tmp_end = None;
    //         }
    //         _ => {
    //             panic!("Tmp_end did not have a value!");
    //         }
    //     }
    //     let mut children = n.children.values_mut().collect();
    //     nodes.append(&mut children);
    // }

    SuffixTree {
        raw_string: String::from(s),
        // root: Box::new(root),
        nodes,
    }
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
        SuffixTree::new("banana");
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
