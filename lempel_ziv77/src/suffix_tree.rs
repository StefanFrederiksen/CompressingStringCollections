use std::borrow::Cow;
use std::rc::Rc;

#[derive(Debug)]
pub struct SuffixTree<'s> {
  raw_text: Cow<'s, str>,
  root: Box<Node>,
}

#[derive(Debug)]
pub struct Node {
  parent: Rc<Option<Node>>,
  children: Vec<Node>, // Lexicographically ordered list/map?
  start: u32,
  end: u32,
  path_len: u32,
}
pub fn init(s: &str) -> SuffixTree {
  let root = Box::new(Node {
    parent: Rc::new(None),
    children: vec![],
    start: 0,
    end: 0,
    path_len: 0,
  });
  SuffixTree {
    raw_text: Cow::Borrowed(s),
    root,
  }
}
