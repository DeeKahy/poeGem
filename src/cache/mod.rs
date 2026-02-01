use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, error, info, warn};

#[derive(Debug, Serialize, Deserialize)]
struct CacheEntry<T> {
    data: T,
    timestamp: DateTime<Utc>,
    ttl_minutes: i64,
}

impl<T> CacheEntry<T> {
    fn new(data: T, ttl_minutes: i64) -> Self {
        Self {
            data,
            timestamp: Utc::now(),
            ttl_minutes,
        }
    }

    fn is_expired(&self) -> bool {
        let expiry_time = self.timestamp + Duration::minutes(self.ttl_minutes);
        Utc::now() > expiry_time
    }

    fn age_minutes(&self) -> i64 {
        let now = Utc::now();
        (now - self.timestamp).num_minutes()
    }
}

pub struct FileCache {
    cache_dir: PathBuf,
}

impl FileCache {
    pub fn new<P: AsRef<Path>>(cache_dir: P) -> Result<Self> {
        let cache_dir = cache_dir.as_ref().to_path_buf();

        // Create cache directory if it doesn't exist
        if !cache_dir.exists() {
            fs::create_dir_all(&cache_dir)
                .with_context(|| format!("Failed to create cache directory: {:?}", cache_dir))?;
            info!("Created cache directory: {:?}", cache_dir);
        }

        Ok(Self { cache_dir })
    }

    pub async fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let file_path = self.get_cache_path(key);

        if !file_path.exists() {
            debug!("Cache miss: {} (file does not exist)", key);
            return Ok(None);
        }

        let content = fs::read_to_string(&file_path)
            .with_context(|| format!("Failed to read cache file: {:?}", file_path))?;

        let cache_entry: CacheEntry<T> = serde_json::from_str(&content)
            .with_context(|| format!("Failed to deserialize cache entry for key: {}", key))?;

        if cache_entry.is_expired() {
            debug!(
                "Cache expired: {} (age: {} minutes, ttl: {} minutes)",
                key,
                cache_entry.age_minutes(),
                cache_entry.ttl_minutes
            );

            // Clean up expired cache file
            if let Err(e) = fs::remove_file(&file_path) {
                warn!("Failed to remove expired cache file {:?}: {}", file_path, e);
            }

            return Ok(None);
        }

        debug!(
            "Cache hit: {} (age: {} minutes, ttl: {} minutes)",
            key,
            cache_entry.age_minutes(),
            cache_entry.ttl_minutes
        );

        Ok(Some(cache_entry.data))
    }

    pub async fn set<T>(&self, key: &str, data: T, ttl_minutes: i64) -> Result<()>
    where
        T: Serialize,
    {
        let cache_entry = CacheEntry::new(data, ttl_minutes);
        let file_path = self.get_cache_path(key);

        let content = serde_json::to_string_pretty(&cache_entry)
            .with_context(|| format!("Failed to serialize cache entry for key: {}", key))?;

        fs::write(&file_path, content)
            .with_context(|| format!("Failed to write cache file: {:?}", file_path))?;

        debug!("Cached data for key: {} (ttl: {} minutes)", key, ttl_minutes);
        Ok(())
    }

    pub async fn delete(&self, key: &str) -> Result<()> {
        let file_path = self.get_cache_path(key);

        if file_path.exists() {
            fs::remove_file(&file_path)
                .with_context(|| format!("Failed to delete cache file: {:?}", file_path))?;
            debug!("Deleted cache entry: {}", key);
        }

        Ok(())
    }

    pub async fn clear(&self) -> Result<u32> {
        let mut count = 0;

        if !self.cache_dir.exists() {
            return Ok(0);
        }

        let entries = fs::read_dir(&self.cache_dir)
            .with_context(|| format!("Failed to read cache directory: {:?}", self.cache_dir))?;

        for entry in entries {
            let entry = entry.with_context(|| "Failed to read directory entry")?;
            let path = entry.path();

            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Err(e) = fs::remove_file(&path) {
                    error!("Failed to remove cache file {:?}: {}", path, e);
                } else {
                    count += 1;
                }
            }
        }

        info!("Cleared {} cache entries", count);
        Ok(count)
    }

    pub async fn cleanup_expired(&self) -> Result<u32> {
        let mut count = 0;

        if !self.cache_dir.exists() {
            return Ok(0);
        }

        let entries = fs::read_dir(&self.cache_dir)
            .with_context(|| format!("Failed to read cache directory: {:?}", self.cache_dir))?;

        for entry in entries {
            let entry = entry.with_context(|| "Failed to read directory entry")?;
            let path = entry.path();

            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                // Try to read and check if expired
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(cache_entry) = serde_json::from_str::<CacheEntry<serde_json::Value>>(&content) {
                        if cache_entry.is_expired() {
                            if let Err(e) = fs::remove_file(&path) {
                                error!("Failed to remove expired cache file {:?}: {}", path, e);
                            } else {
                                count += 1;
                            }
                        }
                    }
                }
            }
        }

        if count > 0 {
            info!("Cleaned up {} expired cache entries", count);
        }

        Ok(count)
    }

    fn get_cache_path(&self, key: &str) -> PathBuf {
        // Sanitize the key to create a valid filename
        let sanitized_key = key
            .chars()
            .map(|c| match c {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => c,
                _ => '_',
            })
            .collect::<String>();

        self.cache_dir.join(format!("{}.json", sanitized_key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_cache_basic_operations() {
        let temp_dir = TempDir::new().unwrap();
        let cache = FileCache::new(temp_dir.path()).unwrap();

        // Test cache miss
        let result: Option<String> = cache.get("test_key").await.unwrap();
        assert!(result.is_none());

        // Test cache set and hit
        cache.set("test_key", "test_value".to_string(), 60).await.unwrap();
        let result: Option<String> = cache.get("test_key").await.unwrap();
        assert_eq!(result, Some("test_value".to_string()));

        // Test cache delete
        cache.delete("test_key").await.unwrap();
        let result: Option<String> = cache.get("test_key").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let temp_dir = TempDir::new().unwrap();
        let cache = FileCache::new(temp_dir.path()).unwrap();

        // Set cache with very short TTL
        cache.set("test_key", "test_value".to_string(), 0).await.unwrap();

        // Should be expired immediately due to 0 TTL
        let result: Option<String> = cache.get("test_key").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_cache_clear() {
        let temp_dir = TempDir::new().unwrap();
        let cache = FileCache::new(temp_dir.path()).unwrap();

        // Set multiple cache entries
        cache.set("key1", "value1".to_string(), 60).await.unwrap();
        cache.set("key2", "value2".to_string(), 60).await.unwrap();

        // Clear cache
        let count = cache.clear().await.unwrap();
        assert_eq!(count, 2);

        // Verify entries are gone
        let result1: Option<String> = cache.get("key1").await.unwrap();
        let result2: Option<String> = cache.get("key2").await.unwrap();
        assert!(result1.is_none());
        assert!(result2.is_none());
    }
}
