// Relative Lempel Ziv Implementation
use std::mem;
use suffix_tree::SuffixTree;

#[derive(Debug, Clone, Copy)]
pub enum EncodePart {
    // The part consists of the (start, end) from
    // the base string. Normally this would be
    // (start, offset) but to work nicely with
    // Rust's slices, the end is used instead.
    Part(usize, usize),
    Byte(u8),
}

pub type EncodedString = Vec<EncodePart>;

#[derive(Debug)]
pub struct RelativeLempelZiv {
    pub base_data: Vec<u8>,
    pub data: Vec<EncodedString>,
}

impl RelativeLempelZiv {
    pub fn encode<T: AsRef<str>>(strings: &[T]) -> Self {
        let (base_string, rest_strings) = base_string(&strings);
        let st = create_suffix_tree(base_string);
        encode_parts(rest_strings, &st)
    }

    pub fn decode(&self) -> Vec<String> {
        internal_decode(self)
    }

    pub fn memory_footprint(&self) -> usize {
        internal_memory_footprint(self)
    }

    // Todo: Maybe this is where the subspace mapping could really come into play
    // Also O-notation of this can't be constant can it..? Has to loop through
    // the EncodedString to find the right position... Even with saving the position
    // space of the EncodePart, it would still at least be binary search.
    pub fn xth_byte(&self) -> u8 {
        0
    }
}

// Todo: Implement serialize
// https://serde.rs/impl-serialize.html
// impl Serialize for RelativeLempelZiv {

// }

// Todo: Find ways to improve the base string finding
// Currently just takes the first..
fn base_string<T: AsRef<str>>(strings: &[T]) -> (&T, &[T]) {
    (&strings[0], &strings[..])
}

fn create_suffix_tree<T: AsRef<str>>(s: T) -> SuffixTree {
    SuffixTree::new(s)
}

fn encode_parts<T: AsRef<str>>(strings: &[T], suffix_tree: &SuffixTree) -> RelativeLempelZiv {
    let mut data = vec![];
    for s in strings {
        let mut encoded_string_list: Vec<EncodePart> = vec![];

        let base_bytes = s.as_ref().as_bytes();
        let mut index = 0;
        while index < base_bytes.len() {
            let next;
            match suffix_tree.longest_substring(&base_bytes[index..]) {
                // If a substring was found, we encode the start and end
                Some((start, end)) => {
                    next = EncodePart::Part(start, end);
                    index += end - start;
                }
                // If no substring was found, we just save the next byte
                // and try again on the remaining
                None => {
                    next = EncodePart::Byte(base_bytes[index]);
                    index += 1;
                }
            };
            encoded_string_list.push(next);
        }
        encoded_string_list.shrink_to_fit();
        data.push(encoded_string_list);
    }

    data.shrink_to_fit();
    RelativeLempelZiv {
        base_data: suffix_tree.string().as_bytes().to_vec(),
        data,
    }
}

fn internal_decode(encoded_data: &RelativeLempelZiv) -> Vec<String> {
    let mut data = Vec::with_capacity(encoded_data.data.len());

    for encoded_string in &encoded_data.data {
        let mut string_parts = vec![];

        for part in encoded_string {
            let mut c = match part {
                EncodePart::Part(start, end) => encoded_data.base_data[*start..*end].to_vec(),
                EncodePart::Byte(c) => vec![*c],
            };
            string_parts.append(&mut c);
        }

        data.push(String::from_utf8(string_parts).unwrap());
    }

    data
}

fn internal_memory_footprint(encoded: &RelativeLempelZiv) -> usize {
    let mut size = 0;

    // "base_data" size
    size += internal_memory_single_list(&encoded.base_data);

    // "data" size (two-dimensional vector)
    size += internal_memory_double_list(&encoded.data);

    size
}

// Using the Copy to ensure it is somewhat
// a primitive type being used, since any non-
// primitive type is heap-allocated
fn internal_memory_single_list<T: Copy>(v: &Vec<T>) -> usize {
    v.capacity() * mem::size_of::<T>()
}

fn internal_memory_double_list<T: Copy>(vv: &Vec<Vec<T>>) -> usize {
    vv.iter().map(|n| internal_memory_single_list(n)).sum()
}

// Priority list:
// 1. Make the Relative Lempel Ziv (RLZ) ✓
// 2. Verify (Property-based testing) ✓
// 3. Benchmark (both time and compression rate)

// Improvements
// 2. If the alphabet is <= 255 letters, is it possible to map the letters into a single byte value, rather than taking multiple bytes?

#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::{quickcheck, Arbitrary, Gen};
    use rand::Rng;

    #[test]
    fn basic() {
        let test_data = vec!["banana", "anaban", "aaa", "nananananabananana"];
        println!("Original: {:?}", test_data);
        let encoded = RelativeLempelZiv::encode(&test_data);
        println!("Encoded: {:?}", encoded);

        let decoded = encoded.decode();
        println!("Decoded:  {:?}", decoded);
    }

    // Quickcheck does not yet allow randomly generating test data
    // for arrays, and seems like they won't until const generics
    // are added (if even then).
    // https://github.com/BurntSushi/quickcheck/issues/187
    // Workaround seems to be to make own array type, and
    // manually implement Arbitrary trait for it.
    #[derive(Debug, Clone)]
    struct ArbArray {
        strings: Vec<String>,
    }

    impl Arbitrary for ArbArray {
        fn arbitrary<G: Gen>(g: &mut G) -> ArbArray {
            // Randomly select size of array [1, 1000)
            let mut rng = rand::thread_rng();
            let size = rng.gen_range(1, 1000);
            let mut strings = Vec::with_capacity(size);
            for _ in 0..size {
                strings.push(String::arbitrary(g));
            }
            ArbArray { strings }
        }
    }

    #[quickcheck]
    fn encode_decode(arr: ArbArray) -> bool {
        arr.strings == RelativeLempelZiv::encode(&arr.strings).decode()
    }
}
