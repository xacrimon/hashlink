pub mod linked_hash_map;
pub mod linked_hash_set;
pub mod lru_cache;
#[cfg(feature = "serde_impl")]
pub mod serde;

use std::collections::hash_map;
use std::hash::BuildHasher;

pub use linked_hash_map::LinkedHashMap;
pub use linked_hash_set::LinkedHashSet;
pub use lru_cache::LruCache;

pub type DefaultHashBuilder = hash_map::RandomState;
pub type DefaultHasher = <hash_map::RandomState as BuildHasher>::Hasher;
