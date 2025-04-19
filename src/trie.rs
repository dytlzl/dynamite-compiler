use std::collections::HashMap;

pub struct Trie {
    double_array: Vec<(usize, usize)>,
}

#[derive(Default)]
struct Node {
    children: HashMap<usize, Node>,
}

const END_SYMBOL: usize = 127;

/// A trie data structure that matches the longest prefix of a given string.
///
/// # Examples
///
/// ```
/// use dynamite_compiler::trie::Trie;
///
/// let mut trie = Trie::new(&["hello", "world"]);
///
/// assert_eq!(trie.matched_length("hello"), 5);
/// assert_eq!(trie.matched_length("world"), 5);
/// assert_eq!(trie.matched_length("hello world"), 5);
/// assert_eq!(trie.matched_length("foo"), 0);
/// ```
impl Trie {
    pub fn new(reserved_symbols: &[&str]) -> Self {
        // Make a double array
        let mut double_array = Vec::with_capacity(1000);
        double_array.extend(std::iter::repeat_n((0, 0), END_SYMBOL * 2 + 1));
        let mut trie = Self { double_array };
        // Make a tree of reserved symbols
        let mut root = Node::default();
        reserved_symbols.iter().for_each(|s| {
            s.chars()
                .fold(&mut root, |current_node, c| {
                    current_node.children.entry(c as usize).or_default()
                })
                .children
                .entry(END_SYMBOL)
                .or_default();
        });
        // Insert a tree into trie
        trie.insert(1, &root);
        trie
    }
    fn insert(&mut self, index: usize, dict: &Node) {
        loop {
            let offset = self.double_array[index].1;
            if index + offset + END_SYMBOL >= self.double_array.len() {
                self.double_array.extend(std::iter::repeat_n(
                    (0, 0),
                    index + offset + END_SYMBOL * 3 - self.double_array.len(),
                ))
            }
            let is_matching = dict
                .children
                .keys()
                .find(|&c_index| self.double_array[index + offset + c_index].0 != 0)
                .map(|_| {
                    self.double_array[index].1 += 1;
                    false
                })
                .unwrap_or(true);
            if is_matching {
                break;
            }
        }
        let offset = self.double_array[index].1;
        dict.children.keys().for_each(|&c_index| {
            self.double_array[index + offset + c_index].0 = index;
        });
        dict.children.iter().for_each(|(c_index, d)| {
            self.insert(index + offset + c_index, d);
        })
    }
    pub fn matched_length(&self, s: &str) -> usize {
        s.char_indices()
            .try_fold(
                (0, 1, self.double_array[1].1),
                |(max_len, current_index, offset), (i, c)| {
                    if (c as usize) >= END_SYMBOL
                        || self.double_array[current_index + offset + (c as usize)].0
                            != current_index
                    {
                        Err((max_len, current_index, offset))
                    } else {
                        let current_index = current_index + offset + (c as usize);
                        let offset = self.double_array[current_index].1;
                        Ok((
                            if self.double_array[current_index + offset + END_SYMBOL].0
                                == current_index
                            {
                                i + 1
                            } else {
                                max_len
                            },
                            current_index,
                            offset,
                        ))
                    }
                },
            )
            .unwrap_or_else(|e| e)
            .0
    }
}
