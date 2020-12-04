use std::collections::BTreeMap;
use std::ops::Range;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

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
  pub end: Arc<AtomicUsize>,

  // The suffix index is where the suffix of the
  // node starts in the original string. Normally
  // this would just be for the leaf nodes, but
  // to support substrings as well, this is
  // changed a little bit.
  // Usage is done via the suffix_range function
  // and then used as a slice of the string.
  // `string[node.suffix_range()]`
  pub suffix_index: Option<usize>,
}

impl Node {
  pub fn new(
    id: NodeId,
    parent: Option<NodeId>,
    suffix_link: Option<NodeId>,
    start: usize,
    global_end: &Arc<AtomicUsize>,
  ) -> Self {
    Node {
      id,
      parent,
      children: BTreeMap::new(),
      suffix_link,
      start,
      end: Arc::clone(global_end),
      suffix_index: None,
    }
  }

  pub fn end(&self) -> usize {
    self.end.load(Ordering::SeqCst)
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

  // A node is the root if it has no parent
  pub fn is_root(&self) -> bool {
    match self.parent {
      None => true,
      Some(_) => false,
    }
  }

  // A node is a leaf if it has no children
  pub fn is_leaf(&self) -> bool {
    self.children.is_empty()
  }

  // An internal node is neither the root nor a leaf
  pub fn is_internal_node(&self) -> bool {
    !self.is_root() && !self.is_leaf()
  }

  pub fn length(&self) -> usize {
    self.end() - self.start
  }

  pub fn suffix_range(&self) -> Range<usize> {
    // The leaf is an additional character, namely the
    // LabelData::Sep value, hence the subtraction.
    let subtraction = if self.is_leaf() { 1 } else { 0 };
    self.suffix_index.unwrap()..self.end() - subtraction
  }
}
