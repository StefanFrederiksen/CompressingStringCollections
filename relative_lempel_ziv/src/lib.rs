// Relative Lempel Ziv Implementation

use suffix_tree::SuffixTree;

#[derive(Debug)]
enum EncodePart {
    // The part consists of the (start, end) from
    // the base string. Normally this would be
    // (start, offset) but to work nicely with
    // Rust's slices, the end is used instead.
    Part(usize, usize),
    Byte(u8),
}

type EncodedString = Vec<EncodePart>;

#[derive(Debug)]
pub struct RelativeLempelZiv {
    // base_string: String,
    base_data: Vec<u8>,
    data: Vec<EncodedString>,
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
}

// Todo: Find ways to improve the base string finding
// Currently just takes the first..
// Also wtf with this function signature
fn base_string<T: AsRef<str>>(strings: &[T]) -> (&T, &[T]) {
    (&strings[0], &strings[1..])
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

        data.push(encoded_string_list);
    }

    RelativeLempelZiv {
        // base_string: String::from(suffix_tree.string()),
        base_data: suffix_tree.string().as_bytes().to_vec(),
        data,
    }
}

// Todo: Lots of cloning going on (.clone() or .to_vec())
// Need to consider if consuming RelativeLempelZiv could avoid it...
fn internal_decode(encoded_data: &RelativeLempelZiv) -> Vec<String> {
    let mut data = vec![String::from_utf8(encoded_data.base_data.clone()).unwrap()];

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

// Priority list:
// 1. Make the Relative Lempel Ziv (RLZ)
// 2. Verify (Property-based testing)
// 3. Benchmark (cargo bench!!!)

// Steps for RLZ
// 1. Find suitable base string
// 2. Create SuffixTree of base string
// 3. Loop through all strings, encoding them (by prefix) to suffixes of the base string
// 4. Profit

// Considerations
// 1. No suitable base string found in collection

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
            let mut strings = vec![];
            for _ in 0..size {
                strings.push(String::arbitrary(g));
            }
            ArbArray { strings }
        }
    }

    #[quickcheck]
    fn encode_decode(arr: ArbArray) -> bool {
        let rlz = RelativeLempelZiv::encode(&arr.strings[..]);
        arr.strings == rlz.decode()
    }
}
