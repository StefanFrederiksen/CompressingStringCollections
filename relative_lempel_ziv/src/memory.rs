struct MemoryUsage {
  compressed_memory: usize,
  random_access_memory: usize,

  raw_size: usize,
}

impl MemoryUsage {
  pub fn new(raw: usize, compressed: usize, random: usize) -> Self {
    MemoryUsage {
      raw_size: raw,
      compressed_memory: compressed,
      random_access_memory: random,
    }
  }

  pub fn total_memory(&self) -> usize {
    self.compressed_memory + self.random_access_memory
  }
}
