// Relative Lempel Ziv Implementation
use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::mem;
use suffix_tree::SuffixTree;

#[derive(Debug, Clone, Copy)]
pub enum EncodePart<U> {
    // The part consists of the (start, end) from
    // the base string. Normally this would be
    // (start, offset) but to work nicely with
    // Rust's slices, the end is used instead.
    Part(U, U),
    Byte(u8),
}

pub type EncodedString<U> = Vec<EncodePart<U>>;

#[derive(Debug)]
pub struct RelativeLempelZiv<U> {
    pub base_data: Vec<u8>,
    pub data: Vec<EncodedString<U>>,
}

impl<U> RelativeLempelZiv<U>
where
    U: Copy + TryFrom<usize> + TryInto<usize>,
    <U as TryFrom<usize>>::Error: fmt::Debug,
    <U as TryInto<usize>>::Error: fmt::Debug,
{
    pub fn encode<T: AsRef<str>>(strings: &[T]) -> Self {
        let base_string = base_string(&strings);
        let st = create_suffix_tree(base_string);
        encode_parts(strings, &st)
    }

    pub fn decode(&self) -> Vec<String> {
        internal_decode(self)
    }

    pub fn memory_footprint(&self) -> usize {
        internal_memory_footprint(self)
    }

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
fn base_string<T: AsRef<str>>(strings: &[T]) -> &T {
    &strings[0]
}

fn create_suffix_tree<T: AsRef<str>>(s: T) -> SuffixTree {
    SuffixTree::new(s)
}

fn encode_parts<U, T>(strings: &[T], suffix_tree: &SuffixTree) -> RelativeLempelZiv<U>
where
    U: TryFrom<usize>,
    <U as TryFrom<usize>>::Error: fmt::Debug,
    T: AsRef<str>,
{
    let mut data = vec![];
    for s in strings {
        let mut encoded_string_list: Vec<EncodePart<U>> = vec![];

        let base_bytes = s.as_ref().as_bytes();
        let mut index = 0;
        while index < base_bytes.len() {
            let next;
            match suffix_tree.longest_substring(&base_bytes[index..]) {
                // If a substring was found, we encode the start and end
                Some((start, end)) => {
                    index += end - start;
                    let start_converted = U::try_from(start).unwrap();
                    let end_converted = U::try_from(end).unwrap();
                    next = EncodePart::Part(start_converted, end_converted);
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

fn internal_decode<U>(encoded_data: &RelativeLempelZiv<U>) -> Vec<String>
where
    U: Copy + TryInto<usize>,
    <U as TryInto<usize>>::Error: fmt::Debug,
{
    let mut data = Vec::with_capacity(encoded_data.data.len());

    for encoded_string in &encoded_data.data {
        let mut string_parts = vec![];

        for part in encoded_string {
            let mut c = match part {
                EncodePart::Part(start, end) => {
                    let _start = (*start).try_into().unwrap();
                    let _end = (*end).try_into().unwrap();

                    encoded_data.base_data[_start.._end].to_vec()
                }
                EncodePart::Byte(c) => vec![*c],
            };
            string_parts.append(&mut c);
        }

        data.push(String::from_utf8(string_parts).unwrap());
    }

    data
}

fn internal_memory_footprint<U: Copy>(encoded: &RelativeLempelZiv<U>) -> usize {
    let mut size = 0;

    // "base_data" size
    size += internal_memory_single_list(&encoded.base_data);

    // "data" size (two-dimensional vector)
    size += internal_memory_double_list(&encoded.data);

    size
}

fn internal_memory_single_list<T: Copy>(v: &Vec<T>) -> usize {
    v.capacity() * mem::size_of::<T>()
}

fn internal_memory_double_list<T: Copy>(vv: &Vec<Vec<T>>) -> usize {
    vv.iter().map(|v| internal_memory_single_list(v)).sum()
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
        let encoded = RelativeLempelZiv::<u8>::encode(&test_data);
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
        arr.strings == RelativeLempelZiv::<u32>::encode(&arr.strings).decode()
    }
}
