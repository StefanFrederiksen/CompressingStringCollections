// Relative Lempel Ziv Implementation
use rand::seq::SliceRandom;
use rayon::prelude::*;
use std::cmp::Ord;
use std::collections::HashSet;
use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::mem;
use std::sync::Mutex;
use suffix_tree::SuffixTree;

// For showing output progress to the cli
use indicatif::{ProgressBar, ProgressStyle};

// For debug
mod analysis;
use analysis::*;

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

// Todo: Debugging
// impl<U> fmt::Debug for RelativeLempelZiv<U>
// where
//     U: fmt::Display,
// {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         // fn fmt_inner(f: &mut fmt::Formatter, s: usize, e: &Vec<EncodedString<U>>) -> fmt::Result {
//         //     writeln!(f, "{}", s)
//         // }
//         // Only output the first 5 data strings...
//         fn fmt_inner<U>(e: &Vec<EncodePart<U>>) -> String
//         where
//             U: fmt::Display,
//         {
//             e.iter()
//                 .map(|v| format!("({}, {})", v.range.0, v.range.1))
//                 .collect::<Vec<_>>()
//                 .join("")
//         }

//         writeln!(f, "Data length: {}", self.data.len());

//         for i in self.data.iter().take(5) {
//             writeln!(f, "{}, {}", i.len(), fmt_inner(&i))?;
//         }

//         // for _ in 0..cmp::min(5, self.data.len()) {
//         // }

//         Ok(())
//     }
// }

impl<U> RelativeLempelZiv<U>
where
    U: Copy + Ord + TryFrom<usize> + TryInto<usize> + Send,
    <U as TryFrom<usize>>::Error: fmt::Debug,
    <U as TryInto<usize>>::Error: fmt::Debug,
{
    pub fn encode_analysis<T: AsRef<str> + Sync>(
        data: &[(T, T)],
        n: Option<Vec<usize>>,
    ) -> (Self, AnalysisResult) {
        let strings: Vec<&str> = data.iter().map(|t| t.0.as_ref()).collect();
        let names: Vec<&str> = data.iter().map(|t| t.1.as_ref()).collect();
        let base_string = base_string(&strings, n);
        let st = create_suffix_tree(base_string);
        let rlz = encode_parts(&strings, &st);

        let mut a_vec = Vec::with_capacity(strings.len());
        for (i, (encoded, name)) in rlz.data.iter().zip(names.iter()).enumerate() {
            let len = encoded.len();
            let c_size = internal_memory_single_list(&encoded);
            let r_size = strings[i].len();
            let analysis = Analysis::new(len, c_size, r_size, name);
            a_vec.push(analysis);
        }

        let analysis_result = AnalysisResult::new(a_vec);
        (rlz, analysis_result)
    }

    pub fn encode<T: AsRef<str> + Sync>(strings: &[T], n: Option<Vec<usize>>) -> Self {
        let spinner_style = ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner} {wide_msg}");

        let pb = ProgressBar::new(1);
        pb.set_style(spinner_style);
        pb.set_message("Finding base string...");
        let base_string = base_string(&strings, n);

        pb.set_message("Creating suffix tree from base string...");
        let st = create_suffix_tree(base_string);

        pb.set_message("Encoding...");
        let res = encode_parts(strings, &st);
        pb.finish_and_clear();
        res
    }

    pub fn encode_reference_merge<T>(strings: &[(T, T)]) -> Self
    where
        T: AsRef<str> + Sync + Eq,
    {
        encode_by_reference_merge(strings)
    }

    pub fn decode(&self) -> Vec<String> {
        internal_decode(self)
    }

    // Gets the x'th byte from the i'th string
    pub fn random_access(&self, i: U, x: U) -> u8 {
        internal_random_access(self, i, x)
    }

    pub fn memory_footprint(&self) -> (usize, usize) {
        internal_memory_footprint(self)
    }

    pub fn uncompressed_size(&self) -> u64 {
        // Need to decode first...
        let decoded = self.decode();
        internal_memory_string_list(&decoded)
    }

    // pub fn compressed_rate(&self) -> f64 {
    //     let (d1, d2) = self.memory_footprint();
    //     let total_size = d1 + d2;
    //     total_size as f64 / self.uncompressed_size() as f64
    // }
}

fn base_string_by_name<T: AsRef<str> + Eq>(strings: &[(T, T)], names: &Vec<String>) -> String {
    let mut ref_str = strings
        .iter()
        .filter(|(_, n)| names.contains(&String::from(n.as_ref())))
        .map(|(s, _)| s.as_ref())
        .collect::<Vec<_>>()
        .join("");
    ref_str.push_str("ACGTN");
    ref_str
}

fn encode_by_reference_merge<U, T>(strings: &[(T, T)]) -> RelativeLempelZiv<U>
where
    U: Copy + Ord + TryFrom<usize> + TryInto<usize> + Send,
    <U as TryFrom<usize>>::Error: fmt::Debug,
    <U as TryInto<usize>>::Error: fmt::Debug,
    T: AsRef<str> + Sync + Eq,
{
    let raw_strings: Vec<&str> = strings.iter().map(|t| t.0.as_ref()).collect();
    let names: Vec<&str> = strings.iter().map(|t| t.1.as_ref()).collect();

    let total_size = internal_memory_string_list(&raw_strings);

    // Initially pick a random reference string
    let mut reference_names: Vec<String> = Vec::new();

    let initial_element = strings.choose(&mut rand::thread_rng()).unwrap();
    reference_names.push(String::from(initial_element.1.as_ref()));

    // Loop until best compression rate is found
    let mut best_compression_rate = 1.0f64;
    let mut i = 0;
    let mut best_rlz = None;
    loop {
        i += 1;

        // This scope is to uninitialize the base_string and st
        // asap because we still do computation after, but they
        // aren't needed for that. Thanks to Rust's borrowing
        // system, they will be removed from memory after the
        // scope ends.
        let rlz: RelativeLempelZiv<U> = {
            let base_string = base_string_by_name(strings, &reference_names);
            let st = create_suffix_tree(base_string);
            encode_parts(&raw_strings, &st)
        };

        let mut a_vec = Vec::with_capacity(strings.len());
        for (i, (encoded, name)) in rlz.data.iter().zip(names.iter()).enumerate() {
            let len = encoded.len();
            let c_size = internal_memory_single_list(&encoded);
            let r_size = raw_strings[i].len();
            let analysis = Analysis::new(len, c_size, r_size, name);
            a_vec.push(analysis);
        }

        let analysis_result = AnalysisResult::new(a_vec);
        let (d1, d2) = rlz.memory_footprint();
        let compressed_rate = (d1 + d2) as f64 / total_size as f64;

        if compressed_rate < best_compression_rate {
            eprintln!(
                "{} < {} in the {}th iteration.",
                compressed_rate, best_compression_rate, i
            );
            best_compression_rate = compressed_rate;

            let worst_ref = String::from(analysis_result.worst_reference_string());
            reference_names.push(worst_ref);

            best_rlz = Some(rlz);
        } else {
            eprintln!(
                "Returning best rate {} with the following strings: {:#?}",
                best_compression_rate, reference_names
            );
            return best_rlz.expect("Tried to return without actually finding an RLZ");
        }
        // (rlz, analysis_result)
    }

    // 1. Pick random reference string at first
    // 2. Encode as usual
    // 3. Find worst encoded other reference string and merge with previous
    // 4. Encode with this instead
    // 5. If performance was better, goto 3
    // 6. If not, go with this.
}

// Todo: Find ways to improve the base string finding
// Todo: Change this to bytes, since that simplifies
// the amount of chars needed.
fn base_string<T: AsRef<str>>(strings: &[T], n: Option<Vec<usize>>) -> String {
    // Select suitable base string
    let base_string = n
        .unwrap_or(vec![0])
        .iter()
        .map(|&x| strings[x].as_ref())
        .collect::<Vec<_>>()
        .join("");
    // let base_string = strings[n.unwrap_or(0)].as_ref();
    // For now assume that reference string contains all chars
    // If this breaks, just ensure ACGTN are there...
    let mut s = String::from(base_string);
    s.push_str("ACGTN");
    return s;

    // Create hash of all current characters
    let mut found_chars = HashSet::new();
    for c in s.chars() {
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

    let mut return_string = String::with_capacity(s.len() + chars_to_add.len());
    return_string.push_str(&s);
    for c in chars_to_add {
        return_string.push(c);
    }

    return_string
}

fn create_suffix_tree<T: AsRef<str>>(s: T) -> SuffixTree {
    SuffixTree::new(s)
}

// fn encode_parts<U, T>(strings: &[T], suffix_tree: &SuffixTree) -> RelativeLempelZiv<U>
// where
//     U: TryFrom<usize>,
//     <U as TryFrom<usize>>::Error: fmt::Debug,
//     T: AsRef<str>,
// {
//     // For io::stderr output of progress
//     let pb = ProgressBar::new(strings.len() as u64);

//     let mut data = vec![];
//     for s in strings {
//         pb.inc(1);

//         let mut encoded_string_list: Vec<EncodePart<U>> = vec![];
//         let mut len = 0;

//         let base_bytes = s.as_ref().as_bytes();
//         let mut index = 0;
//         while index < base_bytes.len() {
//             let len_converted = U::try_from(len).unwrap();
//             let (start, end) = suffix_tree
//                 .longest_substring(&base_bytes[index..])
//                 .expect("Reference string did not contain substring");
//             index += end - start;
//             let start_converted = U::try_from(start).unwrap();
//             let end_converted = U::try_from(end).unwrap();
//             let next = EncodePart {
//                 len: len_converted,
//                 range: (start_converted, end_converted),
//             };
//             len += end - start;
//             encoded_string_list.push(next);
//         }
//         encoded_string_list.shrink_to_fit();
//         data.push(encoded_string_list);
//     }

//     pb.finish_and_clear();

//     data.shrink_to_fit();
//     RelativeLempelZiv {
//         base_data: suffix_tree.string().as_bytes().to_vec(),
//         data,
//     }
// }

fn encode_parts<U, T>(strings: &[T], suffix_tree: &SuffixTree) -> RelativeLempelZiv<U>
where
    U: TryFrom<usize> + Send,
    <U as TryFrom<usize>>::Error: fmt::Debug,
    T: AsRef<str> + Sync,
{
    // For io::stderr output of progress
    let pb = ProgressBar::new(strings.len() as u64);

    // Prep result list
    // Need to insert all empty elements in the list, since
    // with_capacity only ensures that the capacity is there,
    // not that we can insert at position i in the vector.
    let mut mutex_list = Vec::with_capacity(strings.len());
    for _ in strings {
        mutex_list.push(vec![]);
    }

    let data = Mutex::new(mutex_list);

    strings.par_iter().enumerate().for_each(|(i, s)| {
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
        let mut list = data.lock().unwrap();
        list[i] = encoded_string_list;
    });

    pb.finish_and_clear();

    let list = data.into_inner().unwrap();
    RelativeLempelZiv {
        base_data: suffix_tree.string().as_bytes().to_vec(),
        data: list,
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

    data.shrink_to_fit();
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

// --- Memory consumption functions ---
// Computes the memory consumption of a slice of strings
fn internal_memory_string_list<T: AsRef<str>>(v: &[T]) -> u64 {
    v.iter().fold(0, |acc, s| acc + s.as_ref().len() as u64)
}

// Computes the memory consumption of a Vector of stack-allocated elements
fn internal_memory_single_list<T: Copy>(v: &Vec<T>) -> usize {
    v.capacity() * mem::size_of::<T>()
}

// Computes the memory consumption of a Vector of Vectors of stack-allocated elements
fn internal_memory_double_list<T: Copy>(vv: &Vec<Vec<T>>) -> usize {
    vv.iter().map(|v| internal_memory_single_list(v)).sum()
}

// fn total_size_strings<T: AsRef<str>>(strings: &[T]) -> u64 {
//     internal_memory_string_list(&strings)
// }

// // Assumes the first element in the tuple is the data string
// fn total_size_tuples<T: AsRef<str>>(strings: &[(T, T)]) -> u64 {
//     let f = strings.iter().map(|t| t.0.as_ref()).collect::<Vec<_>>();
//     internal_memory_string_list(&f)
// }

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
        let encoded = RelativeLempelZiv::<u8>::encode(&test_data, None);
        println!("Encoded: {:?}", encoded);

        let decoded = encoded.decode();
        println!("Decoded:  {:?}", decoded);
    }

    #[test]
    fn random_access() {
        let test_data = vec!["banana", "ananan", "nananananananv"];
        let encoded = RelativeLempelZiv::<u8>::encode(&test_data, None);

        assert_eq!(b"a"[0], encoded.random_access(1, 0));
        assert_eq!(b"v"[0], encoded.random_access(2, 13));
        assert_eq!(b"n"[0], encoded.random_access(2, 10));
    }

    // If this test fails, just ensure that the part about
    // the base string includes every character. This should
    // ensure that this test passes, since it generates completely
    // random strings.
    #[quickcheck]
    fn quickcheck_encode_decode(xs: Vec<String>) -> TestResult {
        // No point in encoding an empty list, so we discard those
        // test inputs
        if xs.len() == 0 {
            return TestResult::discard();
        }

        let res = xs == RelativeLempelZiv::<u32>::encode(&xs, None).decode();
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
        let encoded = RelativeLempelZiv::<usize>::encode(&xs, None);

        let res = xs[index].as_bytes()[xth] == encoded.random_access(index, xth);
        TestResult::from_bool(res)
    }

    #[test]
    fn testing() {
        println!("Analysis size: {}", mem::size_of::<Analysis>());
    }
}
