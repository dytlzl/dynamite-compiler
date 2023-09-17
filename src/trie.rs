use std::collections::HashMap;

pub struct Trie {
    double_array: Vec<(usize, usize)>,
}

#[derive(Default)]
struct Node {
    children: HashMap<usize, Node>,
}

const END_SYMBOL: usize = 127;

impl Trie {
    pub fn new(reserved_symbols: &[&str]) -> Self {
        // Make a double array
        let mut double_array = Vec::with_capacity(1000);
        double_array.extend(std::iter::repeat((0, 0)).take(END_SYMBOL * 2 + 1));
        let mut trie = Self { double_array };
        // Make a tree of reserved symbols
        let mut root = Node::default();
        for s in reserved_symbols.iter() {
            let mut current_node = &mut root;
            for c in s.chars() {
                let c_num = c as usize;
                current_node = current_node.children.entry(c_num).or_default();
            }
            current_node.children.entry(END_SYMBOL).or_default();
        }
        // Insert a tree into trie
        trie.insert(1, &root);
        trie
    }
    fn insert(&mut self, index: usize, dict: &Node) {
        loop {
            let mut is_matching = true;
            let offset = self.double_array[index].1;
            if index + offset + END_SYMBOL >= self.double_array.len() {
                self.double_array.extend(
                    std::iter::repeat((0, 0))
                        .take(index + offset + END_SYMBOL * 3 - self.double_array.len()),
                )
            }
            for &c_index in dict.children.keys() {
                if self.double_array[index + offset + c_index].0 != 0 {
                    self.double_array[index].1 += 1;
                    is_matching = false;
                    break;
                }
            }
            if is_matching {
                break;
            }
        }
        let offset = self.double_array[index].1;
        for &c_index in dict.children.keys() {
            self.double_array[index + offset + c_index].0 = index;
        }
        for (&c_index, d) in &dict.children {
            self.insert(index + offset + c_index, d);
        }
    }
    pub fn matched_length(&self, s: &str) -> usize {
        let mut max_len = 0;
        let mut current_index = 1;
        let mut offset = self.double_array[current_index].1;
        for (i, c) in s.char_indices() {
            let c_index = c as usize;
            if c_index >= END_SYMBOL
                || self.double_array[current_index + offset + c_index].0 != current_index
            {
                break;
            }
            current_index = current_index + offset + c_index;
            offset = self.double_array[current_index].1;
            if self.double_array[current_index + offset + END_SYMBOL].0 == current_index {
                max_len = i + 1;
            }
        }
        max_len
    }
}
