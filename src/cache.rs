use std::{
    any::Any,
    collections::HashMap,
    sync::{Arc, PoisonError, RwLock},
    time::{Duration, Instant},
};

const CACHE_TTL: Duration = Duration::from_mins(5);

/// A cached value, type-erased so a single cache can hold several concrete types.
///
/// `Send + Sync` so the cache itself is shareable across worker threads.
type CacheValue = Arc<dyn Any + Send + Sync>;

pub struct TTLCache {
    cache: RwLock<HashMap<String, CacheEntry>>,
    ttl: Duration,
}

impl TTLCache {
    pub fn new() -> Self {
        Self::with_ttl(CACHE_TTL)
    }

    pub fn with_ttl(ttl: Duration) -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            ttl,
        }
    }

    /// Returns the live value stored under `key`, if it is present, unexpired and really a `T`.
    ///
    /// An expired entry is dropped as a side effect of the lookup.
    pub fn get<T: Any + Send + Sync>(&self, key: &str) -> Option<Arc<T>> {
        let state = {
            let cache_guard = self.read();
            if let Some(entry) = cache_guard.get(key) {
                if entry.expired(self.ttl) {
                    EntryState::Expired
                } else {
                    EntryState::Hit(entry.value())
                }
            } else {
                EntryState::Miss
            }
        };

        // TODO: Add cache hit / miss telemetry
        match state {
            EntryState::Hit(value) => match value.downcast::<T>() {
                Ok(value) => {
                    tracing::debug!("Cache hit");
                    Some(value)
                }
                Err(_) => {
                    tracing::warn!("Cache entry is not of the requested type");
                    None
                }
            },
            EntryState::Miss => {
                tracing::debug!("Cache miss");
                None
            }
            EntryState::Expired => {
                tracing::debug!("Removing expired key from cache");
                _ = self.remove(key);
                None
            }
        }
    }

    /// Stores `value` under `key`, replacing and returning any entry already there.
    pub fn insert<T: Any + Send + Sync>(&self, key: &str, value: T) -> Option<CacheEntry> {
        let entry = CacheEntry::new(value);
        self.write().insert(key.to_string(), entry)
    }

    fn remove(&self, key: &str) -> Option<CacheEntry> {
        self.write().remove(key)
    }

    fn read(&self) -> std::sync::RwLockReadGuard<'_, HashMap<String, CacheEntry>> {
        self.cache.read().unwrap_or_else(PoisonError::into_inner)
    }

    fn write(&self) -> std::sync::RwLockWriteGuard<'_, HashMap<String, CacheEntry>> {
        self.cache.write().unwrap_or_else(PoisonError::into_inner)
    }
}

impl Default for TTLCache {
    fn default() -> Self {
        Self::new()
    }
}

pub struct CacheEntry {
    created: Instant,
    value: CacheValue,
}

impl CacheEntry {
    fn new<T: Any + Send + Sync>(value: T) -> Self {
        Self {
            created: Instant::now(),
            value: Arc::new(value),
        }
    }

    fn value(&self) -> CacheValue {
        Arc::clone(&self.value)
    }

    fn expired(&self, ttl: Duration) -> bool {
        self.created.elapsed() >= ttl
    }
}

enum EntryState {
    Hit(CacheValue),
    Miss,
    Expired,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_is_shareable_across_threads() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<TTLCache>();
    }

    #[test]
    fn get_returns_a_live_entry() {
        let cache = TTLCache::new();
        cache.insert("greeting", "hello".to_string());

        let value = cache
            .get::<String>("greeting")
            .expect("entry should be live");
        assert_eq!(*value, "hello");
    }

    #[test]
    fn get_returns_none_for_a_missing_entry() {
        let cache = TTLCache::new();
        assert!(cache.get::<String>("nope").is_none());
    }

    #[test]
    fn get_drops_an_expired_entry() {
        let cache = TTLCache::with_ttl(Duration::from_millis(10));
        cache.insert("greeting", "hello".to_string());

        std::thread::sleep(Duration::from_millis(20));

        assert!(cache.get::<String>("greeting").is_none());
        assert!(!cache.read().contains_key("greeting"));
    }

    #[test]
    fn get_returns_none_on_a_type_mismatch() {
        let cache = TTLCache::new();
        cache.insert("count", 42u32);

        assert!(cache.get::<String>("count").is_none());
        // The mismatch does not evict the entry.
        assert_eq!(
            *cache.get::<u32>("count").expect("entry should be live"),
            42
        );
    }

    #[test]
    fn cache_holds_multiple_types_at_once() {
        let cache = TTLCache::new();
        cache.insert("greeting", "hello".to_string());
        cache.insert("count", 42u32);

        assert_eq!(*cache.get::<String>("greeting").unwrap(), "hello");
        assert_eq!(*cache.get::<u32>("count").unwrap(), 42);
    }

    #[test]
    fn insert_replaces_an_existing_entry() {
        let cache = TTLCache::new();
        cache.insert("greeting", "hello".to_string());
        let previous = cache.insert("greeting", "goodbye".to_string());

        assert!(previous.is_some());
        assert_eq!(*cache.get::<String>("greeting").unwrap(), "goodbye");
    }

    #[test]
    fn entry_expiry_tracks_the_ttl() {
        let entry = CacheEntry::new(1u8);

        assert!(!entry.expired(Duration::from_secs(60)));
        assert!(entry.expired(Duration::ZERO));
    }
}
