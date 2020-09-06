use std::cell::Cell;
use std::collections::BTreeMap;
use std::rc::Rc;

use super::label_data::LabelData;

pub type NodeId = usize;
pub type Tree = BTreeMap<LabelData, NodeId>;

#[derive(Debug)]
pub struct Node {
  pub id: NodeId,
  pub parent: Option<NodeId>,
  // Todo: Verify BTreeMap performance, since the
  // implementation is O(n) in Rust, but it utilizes
  // good cache-lookup for speeding up
  // Initial guess is that it would be pretty bad in
  // nodes with a large amount of children, but VERY
  // fast in a node with a limited amount of children
  // (Todo) Also, there can only ever be 257 children
  // on a node, so initial guess is that the BTreeMap
  // will still perform incredibly fast.
  pub children: Tree,
  pub suffix_link: Option<NodeId>,

  pub start: usize,
  pub end: Rc<Cell<usize>>,

  // Todo: Change all of this...
  // This would have been None for non-leaf nodes,
  // and Some for leaf nodes, where the usize
  // represents the index of which the suffix starts
  // in the original string. For example
  // s[suffix_index..] would give the suffix for
  // the particular leaf node. But to properly
  // support finding the largest substring, every
  // node but the root has this index. For a non-
  // leaf node, the substring can be found by
  // s[suffix_index..node.end] (this is identical
  // to the original way of calling it in a leaf
  // node).
  pub suffix_index: Option<usize>,
}

impl Node {
  pub fn new(
    id: NodeId,
    parent: Option<NodeId>,
    suffix_link: Option<NodeId>,
    start: usize,
    global_end: &Rc<Cell<usize>>,
  ) -> Node {
    Node {
      id,
      parent,
      children: BTreeMap::new(),
      suffix_link,
      start,
      end: Rc::clone(global_end),
      suffix_index: None,
    }
  }

  pub fn has_child(&self, b: &LabelData) -> bool {
    self.children.contains_key(b)
  }

  pub fn child(&self, b: &LabelData) -> Option<&NodeId> {
    self.children.get(b)
  }

  pub fn children(&self) -> &Tree {
    &self.children
  }

  pub fn is_root(&self) -> bool {
    match self.parent {
      None => true,
      Some(_) => false,
    }
  }

  // A node is a leaf if it has no children
  // ? Maybe also only if it has the LabelData::Sep
  // as the last part?
  pub fn is_leaf(&self) -> bool {
    self.children.is_empty()
  }

  pub fn is_internal_node(&self) -> bool {
    !self.is_root() && !self.is_leaf()
  }

  pub fn length(&self) -> usize {
    self.end.get() - self.start
  }
}
