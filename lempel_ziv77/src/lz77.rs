use suffix_tree::SuffixTree;

// A list of tuples (start, end) containing the indexes
// for the base string, from which to decode the data
type EncodePart = (usize, usize);
type EncodedString = Vec<EncodePart>;
pub struct EncodedData {
  base_string: String,
  data: Vec<EncodedString>,
}

// Todo: Find ways to improve this
fn base_string(strings: &[String]) -> &str {
  &strings[0]
}

fn create_suffix_tree(s: &str) -> SuffixTree {
  SuffixTree::new(s)
}

fn encode(strings: &[String], suffix_tree: &SuffixTree) -> EncodedData {
  let mut data = vec![];
  for s in strings {
    let encoded_string_list: Vec<(usize, usize)> = vec![];
    let start: usize = 0;
    let end: usize = 0;

    // Byte loop
    for b in s.as_bytes() {}

    data.push(encoded_string_list);
  }

  EncodedData {
    base_string: String::from(suffix_tree.string()),
    data,
  }
}

fn decode() -> bool {
  panic!("Not finished")
}

fn longest_prefix(remaining_bytes: &mut [u8], suffix_tree: &SuffixTree) -> EncodePart {
  let start = 0;
  let end = 0;
  let mut current_node = suffix_tree.root();

  (0, 0)
}

// Priority list:
// 1. Make the LZ77
// 2. Verify
// 3. Benchmark

// Steps for LZ77
// 1. Find suitable base string
// 2. Create SuffixTree of base string
// 3. Loop through all strings, encoding them (by prefix) to suffixes of the base string
// 4. Profit

// Considerations
// 1. No suitable base string found in collection
// Panic for now I think.

// Todo: Testing (including QuickCheck!)
