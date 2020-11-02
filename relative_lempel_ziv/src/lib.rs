// Relative Lempel Ziv Implementation
use std::cmp::Ord;
use std::collections::HashSet;
use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::mem;
use suffix_tree::SuffixTree;

// For showing output progress to the cli
use console::style;
use indicatif::ProgressBar;

#[derive(Debug, Clone, Copy)]
pub struct EncodePart<U> {
    len: U,
    // (start, end)
    // These are the start and end relative to the
    // base string. Normally this would be start
    // and offset but to work nicely with Rust's
    // slices, the end is used instead.
    range: (U, U),
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

    pub fn memory_footprint(&self) -> (usize, usize) {
        internal_memory_footprint(self)
    }

    // Gets the x'th byte from the i'th string
    pub fn random_access(&self, i: U, x: U) -> u8 {
        internal_random_access(self, i, x)
    }
}

// Todo (nice to have): Implement serialize
// https://serde.rs/impl-serialize.html
// impl Serialize for RelativeLempelZiv { }

// Todo: Find ways to improve the base string finding
// Todo: Change this to bytes, since that simplifies
// the amount of chars needed.
fn base_string<T: AsRef<str>>(strings: &[T]) -> String {
    // Select suitable base string
    let base_string = strings[0].as_ref();

    // Create hash of all current characters
    let mut found_chars = HashSet::new();
    for c in base_string.chars() {
        found_chars.insert(c);
    }

    // Iterate through all strings to ensure all characters are covered
    let mut chars_to_add = Vec::new();
    for string in strings {
        for c in string.as_ref().chars() {
            if !found_chars.contains(&c) {
                chars_to_add.push(c);
                found_chars.insert(c);
            }
        }
    }

    let mut return_string = String::with_capacity(base_string.len() + chars_to_add.len());
    return_string.push_str(&base_string);
    for c in chars_to_add {
        return_string.push(c);
    }

    return_string
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
            let len_converted = U::try_from(len).unwrap();
            let (start, end) = suffix_tree
                .longest_substring(&base_bytes[index..])
                .expect("Reference string did not contain substring");
            index += end - start;
            let start_converted = U::try_from(start).unwrap();
            let end_converted = U::try_from(end).unwrap();
            let next = EncodePart {
                len: len_converted,
                range: (start_converted, end_converted),
            };
            len += end - start;
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
            let (start, end) = part.range;
            let start_as_u = start.try_into().unwrap();
            let end_as_u = end.try_into().unwrap();
            let mut c = encoded_data.base_data[start_as_u..end_as_u].to_vec();
            string_parts.append(&mut c);
        }

        data.push(String::from_utf8(string_parts).unwrap());
    }

    data
}

fn internal_memory_footprint<U: Copy>(encoded: &RelativeLempelZiv<U>) -> (usize, usize) {
    // "base_data" size
    let base_data_size = internal_memory_single_list(&encoded.base_data);

    // "data" size (two-dimensional vector)
    let data_size = internal_memory_double_list(&encoded.data);

    // Todo: Debug info
    // println!("Looking into factorization data sizes...");
    // println!("Sequences: {}", encoded.data.len());
    // println!(
    //     "Size of 1 factorization: {}",
    //     mem::size_of::<EncodePart<u32>>(),
    // );
    // println!(
    //     "Factorizations: {}",
    //     encoded.data.iter().map(|v| v.len()).sum::<usize>()
    // );
    // println!(
    //     "Minimum factorization: {}",
    //     encoded.data.iter().map(|v| v.len()).min().unwrap_or(0)
    // );
    // println!(
    //     "Maximum factorization: {}",
    //     encoded.data.iter().map(|v| v.len()).max().unwrap_or(0)
    // );

    // fn mean(list: &[usize]) -> f64 {
    //     let sum: usize = Iterator::sum(list.iter());
    //     sum as f64 / (list.len() as f64)
    // }
    // println!(
    //     "Average factorization: {}",
    //     mean(&encoded.data.iter().map(|v| v.len()).collect::<Vec<_>>())
    // );

    // Return
    (base_data_size, data_size)
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
    let (start, _) = encode_part.range;

    let len_usize = encode_part.len.try_into().unwrap();
    let start_usize = start.try_into().unwrap();
    let pos = start_usize + (x_usize - len_usize);
    rlt.base_data[pos]
}

// Priority list:
// 1. Make the Relative Lempel Ziv (RLZ) ✓
// 2. Verify (Property-based testing) ✓
// 3. Benchmark (both time and compression rate) - not making this automated, but rather from the CLI part. Could include an option that would test a predetermined amount of files and output all at once

// Improvements
// 2. If the alphabet is <= 255 letters, is it possible to map the letters into a single byte value, rather than taking multiple bytes?

#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::{quickcheck, TestResult};
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

    #[quickcheck]
    fn quickcheck_encode_decode(xs: Vec<String>) -> TestResult {
        // No point in encoding an empty list, so we discard those
        // test inputs
        if xs.len() == 0 {
            return TestResult::discard();
        }
        let res = xs == RelativeLempelZiv::<u32>::encode(&xs).decode();
        TestResult::from_bool(res)
    }

    #[quickcheck]
    fn quickcheck_random_access(xs: Vec<String>) -> TestResult {
        if xs.len() == 0 {
            return TestResult::discard();
        }

        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0, xs.len());

        // If the chosen string is an empty string, it
        // has no bytes to validate against, so we skip it
        if xs[index].len() == 0 {
            return TestResult::discard();
        }

        let xth = rng.gen_range(0, xs[index].len());
        let encoded = RelativeLempelZiv::<usize>::encode(&xs);

        let res = xs[index].as_bytes()[xth] == encoded.random_access(index, xth);
        TestResult::from_bool(res)
    }
}
