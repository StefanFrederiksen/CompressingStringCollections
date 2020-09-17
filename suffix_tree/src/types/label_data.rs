use std::cmp::Ordering;
use std::fmt;

// Rejoice for Union Types 🙌
// Enum used for the "characters" in a label
// where the separator will be a unique one
// in *any* string, since it cannot exist from
// the string itself and can thus be used to
// ensure that the Suffix Tree is finished in
// a single pass (satisfying the online condition)
// and converted from an implicit to an explicit
// suffix tree.
#[derive(Copy, Clone)]
pub enum LabelData {
  Byte(u8),
  Sep,
}

// The separator as printed in output
static SEP: &'static str = "<$>";
impl LabelData {
  pub fn new(b: u8) -> Self {
    LabelData::Byte(b)
  }

  pub fn as_readable(&self) -> Vec<u8> {
    match self {
      Self::Byte(b) => vec![*b],
      Self::Sep => SEP.as_bytes().to_vec(),
    }
  }

  pub fn unwrap_byte(&self) -> u8 {
    match self {
      Self::Byte(b) => *b,
      _ => panic!("Could not unwrap LabelData as it was a Separator"),
    }
  }
}

impl PartialEq for LabelData {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (Self::Byte(b1), Self::Byte(b2)) => b1 == b2,
      (Self::Sep, Self::Sep) => true,
      _ => false,
    }
  }
}

impl PartialEq<u8> for LabelData {
  fn eq(&self, other: &u8) -> bool {
    match (self, other) {
      (Self::Byte(b1), b2) => b1 == b2,
      _ => false,
    }
  }
}

impl PartialEq<LabelData> for u8 {
  fn eq(&self, other: &LabelData) -> bool {
    match (self, other) {
      (b1, LabelData::Byte(b2)) => b1 == b2,
      _ => false,
    }
  }
}

impl Eq for LabelData {}

impl Ord for LabelData {
  fn cmp(&self, other: &Self) -> Ordering {
    match (self, other) {
      // The separator is "first" in the ordering, i.e.
      // the lowest value. Otherwise the byte values
      // are just compared to each other
      (Self::Byte(b1), Self::Byte(b2)) => b1.cmp(b2),
      (Self::Sep, Self::Sep) => Ordering::Equal,
      (Self::Byte(_), Self::Sep) => Ordering::Greater,
      (Self::Sep, Self::Byte(_)) => Ordering::Less,
    }
  }
}

impl PartialOrd for LabelData {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl fmt::Debug for LabelData {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    writeln!(f, "LabelData: {:?}", &self.as_readable())
  }
}
