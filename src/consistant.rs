use std::default::Default;
use std::rc::Rc;
use std::iter::Iterator;
use std::collections::hash_map::HashMap;
use crc::crc32::checksum_ieee;

#[derive(Debug)]
pub struct Consistant {
    pub replicas_num: usize,

    circle: HashMap<u32, Rc<String>>,
    members: HashMap<Rc<String>, ()>,
    sorted_keys: Vec<u32>,
}

impl Default for Consistant {
    fn default() -> Consistant {
        Consistant {
            replicas_num: 20,
            circle: HashMap::new(),
            members: HashMap::new(),
            sorted_keys: Vec::new(),
        }
    }
}

impl Consistant {
    pub fn new(replicas_num: usize) -> Self {
        Consistant {
            replicas_num: replicas_num,
            circle: HashMap::new(),
            members: HashMap::new(),
            sorted_keys: Vec::new(),
        }
    }

    pub fn count(&self) -> usize {
        self.members.len()
    }

    pub fn add<S: Into<String>>(&mut self, element: S) {
        let s = &Rc::new(element.into());
        if self.contains(s) {
            return;
        }

        for i in 0..self.replicas_num {
            let sum = checksum_ieee(Self::generate_element_name(s, i).as_bytes());
            self.circle.insert(sum, s.clone());
            self.sorted_keys.push(sum)
        }

        self.members.insert(s.clone(), ());
        self.sorted_keys.sort();
    }

    pub fn get<S: Into<String>>(&self, name: S) -> Option<String> {
        if self.circle.len() == 0 {
            return None;
        }
        let key = self.get_key(checksum_ieee(name.into().as_bytes()));

        match self.circle.get(&key) {
            Some(rc) => Some((**rc).clone()),
            None => unreachable!(),
        }
    }

    pub fn remove<S: Into<String>>(&mut self, name: S) {
        let s = &Rc::new(name.into());
        if !self.contains(s) {
            return;
        }

        for i in 0..self.replicas_num {
            let sum = &checksum_ieee(Self::generate_element_name(s, i).as_bytes());
            self.circle.remove(sum);

            match self.sorted_keys.iter().position(|key| key.eq(sum)) {
                Some(index) => self.sorted_keys.remove(index),
                None => unreachable!(),
            };
        }

        self.members.remove(s);
    }

    #[inline]
    fn contains(&self, name: &Rc<String>) -> bool {
        self.members.contains_key(name)
    }

    #[inline]
    fn get_key(&self, sum: u32) -> u32 {
        for key in &self.sorted_keys {
            if sum < *key {
                return *key;
            }
        }

        self.sorted_keys[0]
    }

    #[inline]
    fn generate_element_name(element: &str, i: usize) -> String {
        String::from(element) + &i.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let consistant = Consistant::default();

        assert_eq!(consistant.replicas_num, 20);
        assert_eq!(consistant.circle.len(), 0);
        assert_eq!(consistant.sorted_keys.len(), 0);
    }

    #[test]
    fn test_new() {
        let consistant = Consistant::new(30);

        assert_eq!(consistant.replicas_num, 30);
        assert_eq!(consistant.circle.len(), 0);
        assert_eq!(consistant.sorted_keys.len(), 0);
    }

    #[test]
    fn test_count() {
        let mut consistant = Consistant::default();
        consistant.add("cacheA");
        consistant.add("cacheB");
        consistant.add("cacheC");
        assert_eq!(consistant.count(), 3);
    }

    #[test]
    fn test_add() {
        let mut consistant = Consistant::default();
        consistant.add("cacheA");
        consistant.add("cacheB");
        consistant.add("cacheC");

        assert_eq!(consistant.circle.len(), 3 * consistant.replicas_num);
        assert_eq!(consistant.sorted_keys.len(), 3 * consistant.replicas_num);
    }

    #[test]
    fn test_contains() {
        let mut consistant = Consistant::default();
        consistant.add("cacheA");

        assert_eq!(consistant.contains(&Rc::new(String::from("cacheA"))), true);
        assert_eq!(consistant.contains(&Rc::new(String::from("cacheB"))), false);
        assert_eq!(consistant.contains(&Rc::new(String::from("CachEa"))), false);
    }

    #[test]
    fn test_get() {
        let mut consistant = Consistant::default();
        consistant.add("cacheA");
        consistant.add("cacheB");
        consistant.add("cacheC");

        assert_eq!(consistant.get("david").unwrap(),
                   consistant.get("david").unwrap());
        assert_eq!(consistant.get("kally").unwrap(),
                   consistant.get("kally").unwrap());
        assert_eq!(consistant.get("jason").unwrap(),
                   consistant.get("jason").unwrap());
    }

    #[test]
    fn test_remove() {
        let mut consistant = Consistant::default();
        consistant.add("cacheA");
        consistant.add("cacheB");
        consistant.add("cacheC");

        consistant.remove("cacheC");
        assert_eq!(consistant.count(), 2);

        assert!(consistant.get("david").unwrap() != String::from("cacheC"));
        assert!(consistant.get("kally").unwrap() != String::from("cacheC"));
        assert!(consistant.get("jason").unwrap() != String::from("cacheC"));
    }
}

