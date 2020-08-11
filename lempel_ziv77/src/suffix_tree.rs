// Some inspiration taken from https://github.com/BurntSushi/suffix
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::rc::Rc;

#[derive(Debug)]
pub struct SuffixTree<'s> {
  raw_string: Cow<'s, str>,
  root: Box<Node>,
}

#[derive(Debug)]
pub struct Node {
  parent: Option<Rc<Node>>,
  // Todo: Verify BTreeMap performance, since the
  // implementation is O(n) in Rust, but it utilizes
  // good cache-lookup for speeding up
  // Initial guess is that it would be pretty bad in
  // nodes with a large amount of children, but VERY
  // fast in a node with a limited amount of children
  children: BTreeMap<char, Box<Node>>,
  suffix_link: Option<Box<Node>>,

  start: u32,
  end: u32,
  path_len: u32,
}

impl<'s> SuffixTree<'s> {
  pub fn new(s: &str) -> SuffixTree {
    init_suffix_tree(s)
  }

  pub fn string(&self) -> &str {
    &self.raw_string
  }

  // IMPORTANT! Untested and might be error prone
  // Gets the label going into the node
  pub fn label_of_node(&self, node: &Node) -> &str {
    &self.raw_string[node.start as usize..node.end as usize]
  }
  // pub fn longest_suffix(&self, s: &str) -> &str // What LZ77 will actually need

  // Other possible methods
  // pub fn contains(&self, s: &str) -> bool
  // pub fn root(&self) -> &Node
}

impl Node {
  // Todo: Needed?
  pub fn root(&self) -> &Node {
    let mut cur = self;
    loop {
      match &cur.parent {
        None => break,
        Some(x) => cur = &x,
      }
    }

    cur
  }
}

// Todo: Issues
// - Unused character used to finish the
//   Suffix Tree, usually $ in literature.
fn init_suffix_tree(s: &str) -> SuffixTree {
  let root = Node {
    parent: None,
    children: BTreeMap::new(),
    suffix_link: None,
    start: 0,
    end: 0,
    path_len: 0,
  };
  // Various control variables
  let mut last_new_node: Option<&Node> = None;
  let mut active_node: Option<&Node> = None;
  let mut active_edge: char = ' ';
  let mut active_length: u32 = 0;
  // let mut active_point: Option<(Box<Node>, char, u32)> = None;
  let mut remaining_suffix_count = 0;
  let mut leaf_end = 0;

  for (i, c) in s.chars().enumerate() {
    // Update leaf_end and increment remaining suffix
    leaf_end = i;
    remaining_suffix_count += 1;

    // Clear last new node
    last_new_node = None;

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

  SuffixTree {
    raw_string: Cow::Borrowed(s),
    root: Box::new(root),
  }
}
