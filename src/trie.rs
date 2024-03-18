use std::{
    collections::{BTreeMap, VecDeque},
    fmt::Display,
};

#[derive(Debug, PartialEq, Eq)]
pub struct Trie<T> {
    value: Option<T>,
    children: BTreeMap<String, Trie<T>>,
}

pub fn translate<I>(input: I, seperator: char) -> VecDeque<String>
where
    I: Into<String>,
{
    VecDeque::from(
        input
            .into()
            .split(seperator)
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
    )
}

impl<T> Trie<T> {
    pub fn new() -> Self {
        Self {
            value: None,
            children: BTreeMap::new(),
        }
    }

    pub fn set(&mut self, mut keys: VecDeque<String>, value: T) {
        if let Some(key) = keys.pop_front() {
            if !self.children.contains_key(&key) {
                self.children.insert(key.clone(), Trie::new());
            }
            self.children.get_mut(&key).unwrap().set(keys, value);
        } else {
            self.value = Some(value);
        }
    }

    pub fn get(&self, mut keys: VecDeque<String>) -> Option<&Trie<T>> {
        let key = keys.pop_front()?;
        if keys.is_empty() {
            return self.children.get(&key);
        } else {
            return self.children.get(&key)?.get(keys);
        }
    }

    pub fn pretty_print<F, B>(&self, f: F, indent: usize)
    where
        F: std::marker::Copy + FnOnce(Option<&T>) -> B,
        B: Display,
    {
        println!("{}", f(self.value.as_ref()));
        for (key, value) in &self.children {
            print!("{}\u{2514} {}: ", " ".repeat(indent + 1), key);
            value.pretty_print(f, indent + 1);
        }
    }
}

#[cfg(test)]
mod test {
    use super::Trie;
    use crate::trie::translate;
    use std::collections::VecDeque;

    #[test]
    fn test() {
        let mut trie = Trie::new();
        let joe = translate("J,o,e", ',');
        let jane = translate("J,a,n,e", ',');
        let john = translate("J,o,h,n", ',');
        trie.set(joe.clone(), 4);
        trie.set(jane.clone(), 2);
        trie.set(john.clone(), 1);

        assert_eq!(trie.get(joe).unwrap().value, Some(&4));
        assert_eq!(trie.get(jane).unwrap().value, Some(&2));
        assert_eq!(trie.get(john).unwrap().value, Some(&1));
        assert_eq!(trie.get(translate("J,o,h,n,n,y", ',')), None);
    }
}
