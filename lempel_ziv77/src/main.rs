// mod lz77;
use std::cell::Cell;
use std::rc::Rc;
use suffix_tree::SuffixTree;

#[derive(Debug)]
struct Node {
    start: usize,
    end: Rc<Cell<usize>>,
}

fn main() {
    // println!("Hello, world!");

    // let string = String::from("banana");
    // println!("{:?}", string.as_bytes());
    // let tree = SuffixTree::new(&string);
    // println!("{:?}", tree);

    // // Get label from suffixtree
    // let mut queue: Vec<_> = tree.root().children().values().rev().collect();
    // while let Some(n) = queue.pop() {
    //     println!("{:?}", tree.label_of_node(&n));
    //     println!("{:?}", tree.label_of_node_formatted(&n));
    //     println!("{} - {}", n.start(), n.end());

    //     let mut children = n.children().values().rev().collect::<Vec<_>>();
    //     queue.append(&mut children);
    // }

    // testing_mutable_referencing();

    let st = SuffixTree::new("banana");
    // let st = SuffixTree::new("xyzxyaxyz$");
    println!("{:?}", st.string().as_bytes());
    println!("{:?}", st);
}

fn testing_mutable_referencing() -> () {
    let global_end = Rc::new(Cell::new(0));
    let n = Node {
        start: 0,
        end: Rc::clone(&global_end),
    };

    let mut nodes = vec![n];
    for i in 0..10 {
        global_end.set(global_end.get() + 1);

        let new_node: Node;
        if i % 2 == 0 {
            new_node = Node {
                start: i,
                end: Rc::clone(&global_end),
            };
        } else {
            new_node = Node {
                start: i,
                end: Rc::new(Cell::new(i)),
            };
        }
        nodes.push(new_node);
        println!("{:?}", nodes.last().unwrap().end);
    }

    println!("{:?}", nodes);
}
