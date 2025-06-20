use std::{
    borrow::Borrow,
    fmt,
    hash::{BuildHasher, Hash, Hasher},
    iter::{Chain, FromIterator},
    ops::{BitAnd, BitOr, BitXor, Sub},
};

use crate::DefaultHashBuilder;
use crate::linked_hash_map::{self, LinkedHashMap, TryReserveError};

pub struct LinkedHashSet<T, S = DefaultHashBuilder> {
    map: LinkedHashMap<T, (), S>,
}

impl<T: Hash + Eq> LinkedHashSet<T, DefaultHashBuilder> {
    pub fn new() -> LinkedHashSet<T, DefaultHashBuilder> {
        LinkedHashSet {
            map: LinkedHashMap::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> LinkedHashSet<T, DefaultHashBuilder> {
        LinkedHashSet {
            map: LinkedHashMap::with_capacity(capacity),
        }
    }
}

impl<T, S> LinkedHashSet<T, S> {
    pub fn capacity(&self) -> usize {
        self.map.capacity()
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            iter: self.map.keys(),
        }
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn drain(&mut self) -> Drain<T> {
        Drain {
            iter: self.map.drain(),
        }
    }

    pub fn clear(&mut self) {
        self.map.clear()
    }

    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&T) -> bool,
    {
        self.map.retain(|k, _| f(k));
    }
}

impl<T, S> LinkedHashSet<T, S>
where
    T: Eq + Hash,
    S: BuildHasher,
{
    pub fn with_hasher(hasher: S) -> LinkedHashSet<T, S> {
        LinkedHashSet {
            map: LinkedHashMap::with_hasher(hasher),
        }
    }

    pub fn with_capacity_and_hasher(capacity: usize, hasher: S) -> LinkedHashSet<T, S> {
        LinkedHashSet {
            map: LinkedHashMap::with_capacity_and_hasher(capacity, hasher),
        }
    }

    pub fn hasher(&self) -> &S {
        self.map.hasher()
    }

    pub fn reserve(&mut self, additional: usize) {
        self.map.reserve(additional)
    }

    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.map.try_reserve(additional)
    }

    pub fn shrink_to_fit(&mut self) {
        self.map.shrink_to_fit()
    }

    pub fn difference<'a>(&'a self, other: &'a LinkedHashSet<T, S>) -> Difference<'a, T, S> {
        Difference {
            iter: self.iter(),
            other,
        }
    }

    pub fn symmetric_difference<'a>(
        &'a self,
        other: &'a LinkedHashSet<T, S>,
    ) -> SymmetricDifference<'a, T, S> {
        SymmetricDifference {
            iter: self.difference(other).chain(other.difference(self)),
        }
    }

    pub fn intersection<'a>(&'a self, other: &'a LinkedHashSet<T, S>) -> Intersection<'a, T, S> {
        Intersection {
            iter: self.iter(),
            other,
        }
    }

    pub fn union<'a>(&'a self, other: &'a LinkedHashSet<T, S>) -> Union<'a, T, S> {
        Union {
            iter: self.iter().chain(other.difference(self)),
        }
    }

    pub fn contains<Q>(&self, value: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.map.contains_key(value)
    }

    pub fn get<Q>(&self, value: &Q) -> Option<&T>
    where
        T: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.map.raw_entry().from_key(value).map(|p| p.0)
    }

    pub fn get_or_insert(&mut self, value: T) -> &T {
        self.map
            .raw_entry_mut()
            .from_key(&value)
            .or_insert(value, ())
            .0
    }

    pub fn get_or_insert_with<Q, F>(&mut self, value: &Q, f: F) -> &T
    where
        T: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
        F: FnOnce(&Q) -> T,
    {
        self.map
            .raw_entry_mut()
            .from_key(value)
            .or_insert_with(|| (f(value), ()))
            .0
    }

    pub fn is_disjoint(&self, other: &LinkedHashSet<T, S>) -> bool {
        self.iter().all(|v| !other.contains(v))
    }

    pub fn is_subset(&self, other: &LinkedHashSet<T, S>) -> bool {
        self.iter().all(|v| other.contains(v))
    }

    pub fn is_superset(&self, other: &LinkedHashSet<T, S>) -> bool {
        other.is_subset(self)
    }

    /// Inserts the given value into the set.
    ///
    /// If the set did not have this value present, inserts it at the *back* of the internal linked
    /// list and returns true, otherwise it moves the existing value to the *back* of the internal
    /// linked list and returns false.
    pub fn insert(&mut self, value: T) -> bool {
        self.map.insert(value, ()).is_none()
    }

    /// Adds the given value to the set, replacing the existing value.
    ///
    /// If a previous value existed, returns the replaced value.  In this case, the value's position
    /// in the internal linked list is *not* changed.
    pub fn replace(&mut self, value: T) -> Option<T> {
        match self.map.entry(value) {
            linked_hash_map::Entry::Occupied(occupied) => Some(occupied.replace_key()),
            linked_hash_map::Entry::Vacant(vacant) => {
                vacant.insert(());
                None
            }
        }
    }

    pub fn remove<Q>(&mut self, value: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.map.remove(value).is_some()
    }

    pub fn take<Q>(&mut self, value: &Q) -> Option<T>
    where
        T: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        match self.map.raw_entry_mut().from_key(value) {
            linked_hash_map::RawEntryMut::Occupied(occupied) => Some(occupied.remove_entry().0),
            linked_hash_map::RawEntryMut::Vacant(_) => None,
        }
    }

    pub fn front(&self) -> Option<&T> {
        self.map.front().map(|(k, _)| k)
    }

    pub fn pop_front(&mut self) -> Option<T> {
        self.map.pop_front().map(|(k, _)| k)
    }

    pub fn back(&self) -> Option<&T> {
        self.map.back().map(|(k, _)| k)
    }

    pub fn pop_back(&mut self) -> Option<T> {
        self.map.pop_back().map(|(k, _)| k)
    }

    pub fn to_front<Q>(&mut self, value: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        match self.map.raw_entry_mut().from_key(value) {
            linked_hash_map::RawEntryMut::Occupied(mut occupied) => {
                occupied.to_front();
                true
            }
            linked_hash_map::RawEntryMut::Vacant(_) => false,
        }
    }

    pub fn to_back<Q>(&mut self, value: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        match self.map.raw_entry_mut().from_key(value) {
            linked_hash_map::RawEntryMut::Occupied(mut occupied) => {
                occupied.to_back();
                true
            }
            linked_hash_map::RawEntryMut::Vacant(_) => false,
        }
    }

    pub fn retain_with_order<F>(&mut self, mut f: F)
    where
        F: FnMut(&T) -> bool,
    {
        self.map.retain_with_order(|k, _| f(k));
    }
}

impl<T: Hash + Eq + Clone, S: BuildHasher + Clone> Clone for LinkedHashSet<T, S> {
    fn clone(&self) -> Self {
        let map = self.map.clone();
        Self { map }
    }
}

impl<T, S> PartialEq for LinkedHashSet<T, S>
where
    T: Eq + Hash,
    S: BuildHasher,
{
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().eq(other)
    }
}

impl<T, S> Hash for LinkedHashSet<T, S>
where
    T: Eq + Hash,
    S: BuildHasher,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        for e in self {
            e.hash(state);
        }
    }
}

impl<T, S> Eq for LinkedHashSet<T, S>
where
    T: Eq + Hash,
    S: BuildHasher,
{
}

impl<T, S> fmt::Debug for LinkedHashSet<T, S>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

impl<T, S> FromIterator<T> for LinkedHashSet<T, S>
where
    T: Eq + Hash,
    S: BuildHasher + Default,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> LinkedHashSet<T, S> {
        let mut set = LinkedHashSet::with_hasher(Default::default());
        set.extend(iter);
        set
    }
}

impl<T, S> Extend<T> for LinkedHashSet<T, S>
where
    T: Eq + Hash,
    S: BuildHasher,
{
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.map.extend(iter.into_iter().map(|k| (k, ())));
    }
}

impl<'a, T, S> Extend<&'a T> for LinkedHashSet<T, S>
where
    T: 'a + Eq + Hash + Copy,
    S: BuildHasher,
{
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        self.extend(iter.into_iter().cloned());
    }
}

impl<T, S> Default for LinkedHashSet<T, S>
where
    S: Default,
{
    fn default() -> LinkedHashSet<T, S> {
        LinkedHashSet {
            map: LinkedHashMap::default(),
        }
    }
}

impl<T, S> BitOr<&LinkedHashSet<T, S>> for &LinkedHashSet<T, S>
where
    T: Eq + Hash + Clone,
    S: BuildHasher + Default,
{
    type Output = LinkedHashSet<T, S>;

    fn bitor(self, rhs: &LinkedHashSet<T, S>) -> LinkedHashSet<T, S> {
        self.union(rhs).cloned().collect()
    }
}

impl<T, S> BitAnd<&LinkedHashSet<T, S>> for &LinkedHashSet<T, S>
where
    T: Eq + Hash + Clone,
    S: BuildHasher + Default,
{
    type Output = LinkedHashSet<T, S>;

    fn bitand(self, rhs: &LinkedHashSet<T, S>) -> LinkedHashSet<T, S> {
        self.intersection(rhs).cloned().collect()
    }
}

impl<T, S> BitXor<&LinkedHashSet<T, S>> for &LinkedHashSet<T, S>
where
    T: Eq + Hash + Clone,
    S: BuildHasher + Default,
{
    type Output = LinkedHashSet<T, S>;

    fn bitxor(self, rhs: &LinkedHashSet<T, S>) -> LinkedHashSet<T, S> {
        self.symmetric_difference(rhs).cloned().collect()
    }
}

impl<T, S> Sub<&LinkedHashSet<T, S>> for &LinkedHashSet<T, S>
where
    T: Eq + Hash + Clone,
    S: BuildHasher + Default,
{
    type Output = LinkedHashSet<T, S>;

    fn sub(self, rhs: &LinkedHashSet<T, S>) -> LinkedHashSet<T, S> {
        self.difference(rhs).cloned().collect()
    }
}

pub struct Iter<'a, K> {
    iter: linked_hash_map::Keys<'a, K, ()>,
}

pub struct IntoIter<K> {
    iter: linked_hash_map::IntoIter<K, ()>,
}

pub struct Drain<'a, K: 'a> {
    iter: linked_hash_map::Drain<'a, K, ()>,
}

pub struct Intersection<'a, T, S> {
    iter: Iter<'a, T>,
    other: &'a LinkedHashSet<T, S>,
}

pub struct Difference<'a, T, S> {
    iter: Iter<'a, T>,
    other: &'a LinkedHashSet<T, S>,
}

pub struct SymmetricDifference<'a, T, S> {
    iter: Chain<Difference<'a, T, S>, Difference<'a, T, S>>,
}

pub struct Union<'a, T, S> {
    iter: Chain<Iter<'a, T>, Difference<'a, T, S>>,
}

impl<'a, T, S> IntoIterator for &'a LinkedHashSet<T, S> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Iter<'a, T> {
        self.iter()
    }
}

impl<T, S> IntoIterator for LinkedHashSet<T, S> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> IntoIter<T> {
        IntoIter {
            iter: self.map.into_iter(),
        }
    }
}

impl<'a, K> Clone for Iter<'a, K> {
    fn clone(&self) -> Iter<'a, K> {
        Iter {
            iter: self.iter.clone(),
        }
    }
}
impl<'a, K> Iterator for Iter<'a, K> {
    type Item = &'a K;

    fn next(&mut self) -> Option<&'a K> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<K> ExactSizeIterator for Iter<'_, K> {}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<&'a T> {
        self.iter.next_back()
    }
}

impl<K: fmt::Debug> fmt::Debug for Iter<'_, K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl<K> Iterator for IntoIter<K> {
    type Item = K;

    fn next(&mut self) -> Option<K> {
        self.iter.next().map(|(k, _)| k)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<K> ExactSizeIterator for IntoIter<K> {}

impl<K> DoubleEndedIterator for IntoIter<K> {
    fn next_back(&mut self) -> Option<K> {
        self.iter.next_back().map(|(k, _)| k)
    }
}

impl<K> Iterator for Drain<'_, K> {
    type Item = K;

    fn next(&mut self) -> Option<K> {
        self.iter.next().map(|(k, _)| k)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<K> DoubleEndedIterator for Drain<'_, K> {
    fn next_back(&mut self) -> Option<K> {
        self.iter.next_back().map(|(k, _)| k)
    }
}

impl<K> ExactSizeIterator for Drain<'_, K> {}

impl<'a, T, S> Clone for Intersection<'a, T, S> {
    fn clone(&self) -> Intersection<'a, T, S> {
        Intersection {
            iter: self.iter.clone(),
            ..*self
        }
    }
}

impl<'a, T, S> Iterator for Intersection<'a, T, S>
where
    T: Eq + Hash,
    S: BuildHasher,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        loop {
            match self.iter.next() {
                None => return None,
                Some(elt) => {
                    if self.other.contains(elt) {
                        return Some(elt);
                    }
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper) = self.iter.size_hint();
        (0, upper)
    }
}

impl<T, S> fmt::Debug for Intersection<'_, T, S>
where
    T: fmt::Debug + Eq + Hash,
    S: BuildHasher,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl<'a, T, S> Clone for Difference<'a, T, S> {
    fn clone(&self) -> Difference<'a, T, S> {
        Difference {
            iter: self.iter.clone(),
            ..*self
        }
    }
}

impl<'a, T, S> Iterator for Difference<'a, T, S>
where
    T: Eq + Hash,
    S: BuildHasher,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        loop {
            match self.iter.next() {
                None => return None,
                Some(elt) => {
                    if !self.other.contains(elt) {
                        return Some(elt);
                    }
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper) = self.iter.size_hint();
        (0, upper)
    }
}

impl<T, S> fmt::Debug for Difference<'_, T, S>
where
    T: fmt::Debug + Eq + Hash,
    S: BuildHasher,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl<'a, T, S> Clone for SymmetricDifference<'a, T, S> {
    fn clone(&self) -> SymmetricDifference<'a, T, S> {
        SymmetricDifference {
            iter: self.iter.clone(),
        }
    }
}

impl<'a, T, S> Iterator for SymmetricDifference<'a, T, S>
where
    T: Eq + Hash,
    S: BuildHasher,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<T, S> fmt::Debug for SymmetricDifference<'_, T, S>
where
    T: fmt::Debug + Eq + Hash,
    S: BuildHasher,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl<'a, T, S> Clone for Union<'a, T, S> {
    fn clone(&self) -> Union<'a, T, S> {
        Union {
            iter: self.iter.clone(),
        }
    }
}

impl<T, S> fmt::Debug for Union<'_, T, S>
where
    T: fmt::Debug + Eq + Hash,
    S: BuildHasher,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl<'a, T, S> Iterator for Union<'a, T, S>
where
    T: Eq + Hash,
    S: BuildHasher,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}
