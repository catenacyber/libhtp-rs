use crate::bstr;
use std::cmp::Ordering;
use std::iter::Iterator;
use std::ops::Index;
use std::slice::SliceIndex;

#[derive(Clone, Debug)]
pub struct Table<T> {
    pub elements: Vec<(bstr::Bstr, T)>,
}

impl<T> Index<usize> for Table<T> {
    type Output = (bstr::Bstr, T);
    fn index(&self, idx: usize) -> &(bstr::Bstr, T) {
        &self.elements[idx]
    }
}

impl<'a, T> IntoIterator for &'a Table<T> {
    type Item = &'a (bstr::Bstr, T);
    type IntoIter = std::slice::Iter<'a, (bstr::Bstr, T)>;

    fn into_iter(self) -> std::slice::Iter<'a, (bstr::Bstr, T)> {
        self.elements.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Table<T> {
    type Item = &'a mut (bstr::Bstr, T);
    type IntoIter = std::slice::IterMut<'a, (bstr::Bstr, T)>;

    fn into_iter(self) -> std::slice::IterMut<'a, (bstr::Bstr, T)> {
        self.elements.iter_mut()
    }
}

impl<T> IntoIterator for Table<T> {
    type Item = (bstr::Bstr, T);
    type IntoIter = std::vec::IntoIter<(bstr::Bstr, T)>;

    fn into_iter(self) -> std::vec::IntoIter<(bstr::Bstr, T)> {
        self.elements.into_iter()
    }
}

impl<T> Table<T> {
    /// Make a new owned Table with given capacity
    pub fn with_capacity(size: usize) -> Self {
        Self {
            elements: Vec::with_capacity(size),
        }
    }

    /// Add a new tuple (key, item) to the table
    pub fn add(&mut self, key: bstr::Bstr, item: T) {
        self.elements.push((key, item));
    }

    pub fn get<I>(&self, index: I) -> Option<&I::Output>
    where
        I: SliceIndex<[(bstr::Bstr, T)]>,
    {
        self.elements.get(index)
    }

    pub fn get_mut<I>(&mut self, index: I) -> Option<&mut I::Output>
    where
        I: SliceIndex<[(bstr::Bstr, T)]>,
    {
        self.elements.get_mut(index)
    }

    /// Search the table for the first tuple with a key matching the given slice, ingnoring ascii case in self
    ///
    /// Returns None if no match is found.
    pub fn get_nocase<K: AsRef<[u8]>>(&self, key: K) -> Option<&(bstr::Bstr, T)> {
        self.elements
            .iter()
            .find(|x| x.0.cmp_nocase(key.as_ref()) == Ordering::Equal)
    }

    /// Search the table for the first tuple with a key matching the given slice, ingnoring ascii case in self
    ///
    /// Returns None if no match is found.
    pub fn get_nocase_mut<K: AsRef<[u8]>>(&mut self, key: K) -> Option<&mut (bstr::Bstr, T)> {
        self.elements
            .iter_mut()
            .find(|x| x.0.cmp_nocase(key.as_ref()) == Ordering::Equal)
    }

    /// Search the table for the first tuple with a tuple key matching the given slice, ignoring ascii case and any zeros in self
    ///
    /// Returns None if no match is found.
    pub fn get_nocase_nozero<K: AsRef<[u8]>>(&self, key: K) -> Option<&(bstr::Bstr, T)> {
        self.elements
            .iter()
            .find(|x| x.0.cmp_nocase_nozero(key.as_ref()) == Ordering::Equal)
    }

    /// Search the table for the first tuple with a tuple key matching the given slice, ignoring ascii case and any zeros in self
    ///
    /// Returns None if no match is found.
    pub fn get_nocase_nozero_mut<K: AsRef<[u8]>>(
        &mut self,
        key: K,
    ) -> Option<&mut (bstr::Bstr, T)> {
        self.elements
            .iter_mut()
            .find(|x| x.0.cmp_nocase_nozero(key.as_ref()) == Ordering::Equal)
    }

    /// Returns the number of elements in the table
    pub fn size(&self) -> usize {
        self.elements.len()
    }
}

// Tests

#[test]
fn Add() {
    let mut t = Table::with_capacity(1);
    let mut k = bstr::Bstr::from("Key");
    assert_eq!(0, t.size());
    t.add(k, "Value1");
    assert_eq!(1, t.size());
    k = bstr::Bstr::from("AnotherKey");
    t.add(k, "Value2");
    assert_eq!(2, t.size());
}

#[test]
fn GetNoCase() {
    let mut t = Table::with_capacity(2);
    let mut k = bstr::Bstr::from("Key1");
    t.add(k, "Value1");
    k = bstr::Bstr::from("KeY2");
    t.add(k, "Value2");

    let mut result = t.get_nocase("KEY1");
    let mut res = result.unwrap();
    assert_eq!(Ordering::Equal, res.0.cmp("Key1"));
    assert_eq!("Value1", res.1);

    result = t.get_nocase("keY1");
    res = result.unwrap();
    assert_eq!(Ordering::Equal, res.0.cmp("Key1"));
    assert_eq!("Value1", res.1);

    result = t.get_nocase("key2");
    res = result.unwrap();
    assert_eq!(Ordering::Equal, res.0.cmp("KeY2"));
    assert_eq!("Value2", res.1);

    result = t.get_nocase("NotAKey");
    assert!(result.is_none());
}

#[test]
fn GetNocaseNozero() {
    let mut t = Table::with_capacity(2);
    let mut k = bstr::Bstr::from("K\x00\x00\x00\x00ey\x001");
    t.add(k, "Value1");
    k = bstr::Bstr::from("K\x00e\x00\x00Y2");
    t.add(k, "Value2");

    let mut result = t.get_nocase_nozero("key1");
    let mut res = result.unwrap();
    assert_eq!(Ordering::Equal, res.0.cmp("K\x00\x00\x00\x00ey\x001"));
    assert_eq!("Value1", res.1);

    result = t.get_nocase_nozero("KeY1");
    res = result.unwrap();
    assert_eq!(Ordering::Equal, res.0.cmp("K\x00\x00\x00\x00ey\x001"));
    assert_eq!("Value1", res.1);

    result = t.get_nocase_nozero("KEY2");
    res = result.unwrap();
    assert_eq!(Ordering::Equal, res.0.cmp("K\x00e\x00\x00Y2"));
    assert_eq!("Value2", res.1);

    result = t.get_nocase("key1");
    assert!(result.is_none());
}

#[test]
fn IndexAccess() {
    let mut t = Table::with_capacity(2);
    let mut k = bstr::Bstr::from("Key1");
    t.add(k, "Value1");
    k = bstr::Bstr::from("KeY2");
    t.add(k, "Value2");

    let res = &t[1];
    assert_eq!(Ordering::Equal, res.0.cmp("KeY2"));
    assert_eq!("Value2", res.1);
    assert_eq!("Value2", t.get(1).unwrap().1);

    let mut res_mut = t.get_mut(1).unwrap();
    res_mut.1 = "Value3";
    assert_eq!("Value3", t.get(1).unwrap().1);
}

#[test]
fn Iterators() {
    let mut table = Table::with_capacity(2);
    table.add("1".into(), "abc".to_string());
    table.add("2".into(), "def".to_string());

    let mut iter_ref: std::slice::Iter<(bstr::Bstr, String)> = (&table).into_iter();
    let (key1, _): &(bstr::Bstr, String) = iter_ref.next().unwrap();
    assert_eq!(key1, &"1");
    assert_eq!(table.get_nocase("1").unwrap().1, "abc");

    let mut iter_mut_ref: std::slice::IterMut<(bstr::Bstr, String)> = (&mut table).into_iter();
    let (key1, ref mut val1): &mut (bstr::Bstr, String) = iter_mut_ref.next().unwrap();
    *val1 = "xyz".to_string();
    assert_eq!(key1, &"1");
    assert_eq!(table.get_nocase("1").unwrap().1, "xyz");

    let mut iter_owned: std::vec::IntoIter<(bstr::Bstr, String)> = table.into_iter();
    let (key1, val1) = iter_owned.next().unwrap();
    assert_eq!(key1, "1");
    assert_eq!(val1, "xyz");
}