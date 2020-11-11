// Factorization size
// Compressed rate (compressized size and raw size)
// Assembly name

use std::fmt;

pub struct Analysis {
  len: usize,
  c_size: usize,
  r_size: usize,
  name: String,
}

impl Analysis {
  pub fn new<T: AsRef<str>>(len: usize, c_size: usize, r_size: usize, name: T) -> Self {
    Analysis {
      len,
      c_size,
      r_size,
      name: String::from(name.as_ref()),
    }
  }

  pub fn compressed_rate(&self) -> f64 {
    self.c_size as f64 / self.r_size as f64
  }
}

impl fmt::Display for Analysis {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
      f,
      "{:.4}, {}, {}, {}, {}",
      self.compressed_rate(),
      self.c_size,
      self.r_size,
      self.len,
      self.name
    )
  }
}
