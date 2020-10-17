// Relative Lempel Ziv Implementation
use std::cmp::Ord;
use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::mem;
use suffix_tree::SuffixTree;

// For showing output progress to the cli
use console::style;
use indicatif::ProgressBar;

#[derive(Debug, Clone, Copy)]
enum EncodeType<U> {
    // (len, start, end)
    // The (start, end) part consists of the start
    // and end relative to the base string. Normally
    // this would be start and offset but to work
    // nicely with Rust's slices, the end is used
    // instead.
    Part(U, U),
    Byte(u8),
}

#[derive(Debug, Clone, Copy)]
pub struct EncodePart<U> {
    len: U,
    encode_type: EncodeType<U>,
}

pub type EncodedString<U> = Vec<EncodePart<U>>;

#[derive(Debug)]
pub struct RelativeLempelZiv<U> {
    pub base_data: Vec<u8>,
    pub data: Vec<EncodedString<U>>,
}

impl<U> RelativeLempelZiv<U>
where
    U: Copy + Ord + TryFrom<usize> + TryInto<usize>,
    <U as TryFrom<usize>>::Error: fmt::Debug,
    <U as TryInto<usize>>::Error: fmt::Debug,
{
    pub fn encode<T: AsRef<str>>(strings: &[T]) -> Self {
        eprintln!("{} Finding base string...", style("[1/3]").bold().dim());
        let base_string = base_string(&strings);

        eprintln!(
            "{} Creating suffix tree from base string...",
            style("[2/3]").bold().dim()
        );
        let st = create_suffix_tree(base_string);

        eprintln!("{} Encoding...", style("[3/3]").bold().dim());
        encode_parts(strings, &st)
    }

    pub fn decode(&self) -> Vec<String> {
        internal_decode(self)
    }

    pub fn memory_footprint(&self) -> usize {
        internal_memory_footprint(self)
    }

    // Gets the x'th byte from the i'th string
    pub fn random_access(&self, i: U, x: U) -> u8 {
        internal_random_access(self, i, x)
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
    // For io::stderr output of progress
    let pb = ProgressBar::new(strings.len() as u64);

    let mut data = vec![];
    for s in strings {
        pb.inc(1);

        let mut encoded_string_list: Vec<EncodePart<U>> = vec![];
        let mut len = 0;

        let base_bytes = s.as_ref().as_bytes();
        let mut index = 0;
        while index < base_bytes.len() {
            let next;
            let len_converted = U::try_from(len).unwrap();
            match suffix_tree.longest_substring(&base_bytes[index..]) {
                // If a substring was found, we encode the start and end
                Some((start, end)) => {
                    index += end - start;
                    let start_converted = U::try_from(start).unwrap();
                    let end_converted = U::try_from(end).unwrap();
                    next = EncodePart {
                        len: len_converted,
                        encode_type: EncodeType::Part(start_converted, end_converted),
                    };
                    len += end - start;
                }
                // If no substring was found, we just save the next byte
                // and try again on the remaining
                None => {
                    next = EncodePart {
                        len: len_converted,
                        encode_type: EncodeType::Byte(base_bytes[index]),
                    };
                    index += 1;
                    len += 1;
                }
            };
            encoded_string_list.push(next);
        }
        encoded_string_list.shrink_to_fit();
        data.push(encoded_string_list);
    }

    pb.finish_and_clear();

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
            let mut c = match part.encode_type {
                EncodeType::Part(start, end) => {
                    let start_as_u = start.try_into().unwrap();
                    let end_as_u = end.try_into().unwrap();

                    encoded_data.base_data[start_as_u..end_as_u].to_vec()
                }
                EncodeType::Byte(c) => vec![c],
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

fn internal_random_access<U>(rlt: &RelativeLempelZiv<U>, i: U, x: U) -> u8
where
    U: Copy + Ord + TryInto<usize>,
    <U as TryInto<usize>>::Error: fmt::Debug,
{
    // Converts i and x into usize for later
    let i_usize = i.try_into().unwrap();
    let x_usize = x.try_into().unwrap();

    let encoded_string: &EncodedString<U> = &rlt.data[i_usize];

    // Binary search on the string to find the corresponding
    // encode part that encompasses the x'th byte
    let matching_element = encoded_string.binary_search_by(|probe| probe.len.cmp(&x));

    // If the binary search does not find the exact element,
    // it returns the next position, where it could be inserted.
    // So because we want the previous one, we can just cover
    // this use-case via a match.
    let index = match matching_element {
        Ok(i) => i,
        Err(i) => i - 1,
    };

    let encode_part = encoded_string[index];
    match encode_part.encode_type {
        // The x'th byte is found via the difference in the
        // length of the Part and the requested x, and is
        // then found from the reference string by adding
        // start to it.
        EncodeType::Part(start, _) => {
            let len_usize = encode_part.len.try_into().unwrap();
            let start_usize = start.try_into().unwrap();
            let pos = start_usize + (x_usize - len_usize);
            rlt.base_data[pos]
        }
        EncodeType::Byte(byte) => byte,
    }
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

    #[test]
    fn random_access() {
        let test_data = vec!["banana", "ananan", "nananananananv"];
        let encoded = RelativeLempelZiv::<u8>::encode(&test_data);

        assert_eq!(b"a"[0], encoded.random_access(1, 0));
        assert_eq!(b"v"[0], encoded.random_access(2, 13));
        assert_eq!(b"n"[0], encoded.random_access(2, 10));
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
    fn quickcheck_encode_decode(arr: ArbArray) -> bool {
        arr.strings == RelativeLempelZiv::<u32>::encode(&arr.strings).decode()
    }

    #[quickcheck]
    fn quickcheck_random_access(arr: ArbArray) -> bool {
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0, arr.strings.len());

        // If the chosen string is an empty string, it
        // has no bytes to validate against, so we skip it
        if arr.strings[index].len() == 0 {
            return true;
        }

        let xth = rng.gen_range(0, arr.strings[index].len());
        let encoded = RelativeLempelZiv::<usize>::encode(&arr.strings);

        arr.strings[index].as_bytes()[xth] == encoded.random_access(index, xth)
    }
}
