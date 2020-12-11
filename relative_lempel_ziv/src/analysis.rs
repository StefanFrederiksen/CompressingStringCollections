// Factorization size
// Compressed rate (compressized size and raw size)
// Assembly name

use std::fmt;

pub struct AnalysisResult {
  pub list: Vec<Analysis>,
}

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

impl AnalysisResult {
  pub fn new(mut list: Vec<Analysis>) -> Self {
    list.sort_by(|a, b| {
      b.compressed_rate()
        .partial_cmp(&a.compressed_rate())
        .unwrap()
    });
    AnalysisResult { list }
  }

  // Reference is valid as long as self is valid (the list is ummutable after initialization)
  // Assumes the vector is sorted from init (which it should be since it's sorted on init and immutable)
  pub fn worst_reference_string<'s>(&'s self) -> &'s str {
    &self.list[0].name
  }
}

impl fmt::Display for AnalysisResult {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    for s in self.list.iter() {
      write!(f, "{}\n", s)?;
    }
    Ok(())
  }
}
