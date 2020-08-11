use unicode_segmentation::UnicodeSegmentation;
mod suffix_tree;

fn main() {
    println!("Hello, world!");

    // To work in graphemes or not to...
    // Need to analyze running time if so, and it probably wouldn't
    // be that much of an overhead if it was just regular chars instead.
    // Might run into some issues with invalid strings, but it might not matter?
    let s = "नमस्ते";
    let graphemes = UnicodeSegmentation::graphemes(s, true).collect::<Vec<&str>>();
    println!("{:?}", graphemes);
}
