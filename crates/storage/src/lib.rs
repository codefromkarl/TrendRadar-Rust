//! 存储抽象：内存与文件 SQLite 仓储。

use rusqlite::{Connection, params};
use std::collections::BTreeMap;
use std::path::Path;
use trendradar_domain::TrendRadarError;
use trendradar_domain::{NewsItem, Result};

/// 新闻存储接口。
pub trait NewsRepository {
    /// 保存一条新闻。
    fn save_news(&mut self, item: NewsItem) -> Result<()>;

    /// 批量保存多条新闻。
    fn save_news_batch(&mut self, items: &[NewsItem]) -> Result<()> {
        for item in items {
            self.save_news(item.clone())?;
        }
        Ok(())
    }

    /// 列出所有已保存新闻。
    fn list_news(&self) -> Result<Vec<NewsItem>>;
}

/// SQLite 新闻仓储。
pub struct SqliteNewsRepository {
    connection: Connection,
}

impl SqliteNewsRepository {
    /// 创建一个内存 SQLite 仓储。
    pub fn in_memory() -> Result<Self> {
        let connection =
            Connection::open_in_memory().map_err(|error| TrendRadarError::Storage {
                message: format!("failed to open in-memory sqlite database: {error}"),
            })?;
        let repository = Self { connection };
        repository.initialize_schema()?;
        Ok(repository)
    }

    /// 打开或创建文件 SQLite 仓储。
    ///
    /// 若文件不存在会自动创建；若已存在则复用已有数据。
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| TrendRadarError::Storage {
                message: format!(
                    "failed to create database directory {}: {error}",
                    parent.display()
                ),
            })?;
        }
        let connection = Connection::open(path).map_err(|error| TrendRadarError::Storage {
            message: format!(
                "failed to open sqlite database at {}: {error}",
                path.display()
            ),
        })?;
        let repository = Self { connection };
        repository.initialize_schema()?;
        Ok(repository)
    }

    fn initialize_schema(&self) -> Result<()> {
        self.connection
            .execute(
                "CREATE TABLE IF NOT EXISTS news_items (
                    source_id TEXT NOT NULL,
                    title TEXT NOT NULL,
                    rank INTEGER NOT NULL,
                    PRIMARY KEY (source_id, title)
                )",
                [],
            )
            .map(|_| ())
            .map_err(|error| TrendRadarError::Storage {
                message: format!("failed to initialize sqlite schema: {error}"),
            })
    }
}

impl NewsRepository for SqliteNewsRepository {
    fn save_news(&mut self, item: NewsItem) -> Result<()> {
        self.connection
            .execute(
                "INSERT INTO news_items (source_id, title, rank)
                 VALUES (?1, ?2, ?3)
                 ON CONFLICT(source_id, title)
                 DO UPDATE SET rank = MIN(news_items.rank, excluded.rank)",
                params![item.source_id, item.title, item.rank],
            )
            .map(|_| ())
            .map_err(|error| TrendRadarError::Storage {
                message: format!("failed to save news item: {error}"),
            })
    }

    fn save_news_batch(&mut self, items: &[NewsItem]) -> Result<()> {
        let transaction =
            self.connection
                .transaction()
                .map_err(|error| TrendRadarError::Storage {
                    message: format!("failed to open sqlite transaction: {error}"),
                })?;

        {
            let mut statement = transaction
                .prepare(
                    "INSERT INTO news_items (source_id, title, rank)
                     VALUES (?1, ?2, ?3)
                     ON CONFLICT(source_id, title)
                     DO UPDATE SET rank = MIN(news_items.rank, excluded.rank)",
                )
                .map_err(|error| TrendRadarError::Storage {
                    message: format!("failed to prepare batch insert statement: {error}"),
                })?;

            for item in items {
                statement
                    .execute(params![&item.source_id, &item.title, item.rank])
                    .map_err(|error| TrendRadarError::Storage {
                        message: format!("failed to save batch news item: {error}"),
                    })?;
            }
        }

        transaction
            .commit()
            .map_err(|error| TrendRadarError::Storage {
                message: format!("failed to commit sqlite transaction: {error}"),
            })
    }

    fn list_news(&self) -> Result<Vec<NewsItem>> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT title, source_id, rank
                 FROM news_items
                 ORDER BY rank ASC, source_id ASC, title ASC",
            )
            .map_err(|error| TrendRadarError::Storage {
                message: format!("failed to prepare sqlite query: {error}"),
            })?;

        let rows = statement
            .query_map([], |row| {
                Ok(NewsItem {
                    title: row.get(0)?,
                    source_id: row.get(1)?,
                    rank: row.get(2)?,
                })
            })
            .map_err(|error| TrendRadarError::Storage {
                message: format!("failed to query stored news: {error}"),
            })?;

        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|error| TrendRadarError::Storage {
                message: format!("failed to decode stored news: {error}"),
            })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MockRemoteFailureMode {
    None,
    Save,
    List,
}

/// 远程对象存储 mock 仓储。
///
/// 该类型仅用于远程对象存储契约演进阶段的测试和原型验证：
///
/// - 复用与 SQLite 一致的去重和排序语义
/// - 模拟远程后端的写入 / 读取错误
/// - 不承担真实网络 IO
pub struct MockRemoteNewsRepository {
    #[allow(dead_code)]
    prefix: String,
    items: BTreeMap<(String, String), u32>,
    failure_mode: MockRemoteFailureMode,
}

impl MockRemoteNewsRepository {
    /// 创建一个 mock 远程仓储。
    #[must_use]
    pub fn new(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
            items: BTreeMap::new(),
            failure_mode: MockRemoteFailureMode::None,
        }
    }

    /// 创建一个写入失败的 mock 远程仓储。
    #[must_use]
    pub fn fail_on_save(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
            items: BTreeMap::new(),
            failure_mode: MockRemoteFailureMode::Save,
        }
    }

    /// 创建一个读取失败的 mock 远程仓储。
    #[must_use]
    pub fn fail_on_list(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
            items: BTreeMap::new(),
            failure_mode: MockRemoteFailureMode::List,
        }
    }
}

impl NewsRepository for MockRemoteNewsRepository {
    fn save_news(&mut self, item: NewsItem) -> Result<()> {
        if self.failure_mode == MockRemoteFailureMode::Save {
            return Err(TrendRadarError::Storage {
                message: "mock remote storage save failed".to_owned(),
            });
        }

        let key = (item.source_id, item.title);
        self.items
            .entry(key)
            .and_modify(|rank| *rank = (*rank).min(item.rank))
            .or_insert(item.rank);
        Ok(())
    }

    fn list_news(&self) -> Result<Vec<NewsItem>> {
        if self.failure_mode == MockRemoteFailureMode::List {
            return Err(TrendRadarError::Storage {
                message: "mock remote storage list failed".to_owned(),
            });
        }

        let mut items: Vec<NewsItem> = self
            .items
            .iter()
            .map(|((source_id, title), rank)| NewsItem {
                title: title.clone(),
                source_id: source_id.clone(),
                rank: *rank,
            })
            .collect();
        items.sort_by(|left, right| {
            left.rank
                .cmp(&right.rank)
                .then_with(|| left.source_id.cmp(&right.source_id))
                .then_with(|| left.title.cmp(&right.title))
        });
        Ok(items)
    }
}

#[cfg(test)]
mod tests {
    use super::{MockRemoteNewsRepository, NewsRepository, SqliteNewsRepository};
    use std::error::Error;
    use std::fs::read_to_string;
    use trendradar_domain::NewsItem;

    #[test]
    fn sqlite_repository_roundtrips_fixture_items() -> Result<(), Box<dyn Error>> {
        let fixture_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixtures/system/storage/news-roundtrip-input.json"
        );
        let contents = read_to_string(fixture_path)?;
        let items: Vec<NewsItem> = serde_json::from_str(&contents)?;
        let mut repository = SqliteNewsRepository::in_memory()?;

        for item in items {
            repository.save_news(item)?;
        }

        let stored = repository.list_news()?;
        assert_eq!(stored.len(), 2);
        assert_eq!(stored[0].title, "Rust 1.85.0 released");
        assert_eq!(stored[0].rank, 1);
        assert_eq!(stored[1].source_id, "community-hotlist");
        Ok(())
    }

    #[test]
    fn sqlite_repository_batch_write_preserves_dedup_semantics() -> Result<(), Box<dyn Error>> {
        let mut repository = SqliteNewsRepository::in_memory()?;
        let items = vec![
            NewsItem {
                title: "Rust 1.85.0 released".to_owned(),
                source_id: "weibo".to_owned(),
                rank: 12,
            },
            NewsItem {
                title: "Rust 1.85.0 released".to_owned(),
                source_id: "weibo".to_owned(),
                rank: 3,
            },
            NewsItem {
                title: "TrendRadar migration plan updated".to_owned(),
                source_id: "rust-blog".to_owned(),
                rank: 8,
            },
        ];

        repository.save_news_batch(&items)?;

        let stored = repository.list_news()?;
        assert_eq!(stored.len(), 2);
        assert_eq!(stored[0].title, "Rust 1.85.0 released");
        assert_eq!(stored[0].rank, 3);
        assert_eq!(stored[1].title, "TrendRadar migration plan updated");
        Ok(())
    }

    #[test]
    fn sqlite_repository_keeps_best_rank_for_duplicate_titles() -> Result<(), Box<dyn Error>> {
        let mut repository = SqliteNewsRepository::in_memory()?;
        repository.save_news(NewsItem {
            title: "Rust 1.85.0 released".to_owned(),
            source_id: "github-trending".to_owned(),
            rank: 5,
        })?;
        repository.save_news(NewsItem {
            title: "Rust 1.85.0 released".to_owned(),
            source_id: "github-trending".to_owned(),
            rank: 2,
        })?;

        let stored = repository.list_news()?;
        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0].rank, 2);
        Ok(())
    }

    #[test]
    fn sqlite_repository_starts_empty() -> Result<(), Box<dyn Error>> {
        let repository = SqliteNewsRepository::in_memory()?;

        let stored = repository.list_news()?;

        assert!(stored.is_empty());
        Ok(())
    }

    #[test]
    fn sqlite_repository_persists_to_file() -> Result<(), Box<dyn Error>> {
        let dir = tempfile::tempdir()?;
        let db_path = dir.path().join("test.db");

        {
            let mut repo = SqliteNewsRepository::open(&db_path)?;
            repo.save_news(NewsItem {
                title: "Persisted news".to_owned(),
                source_id: "test".to_owned(),
                rank: 1,
            })?;
        }

        let repo = SqliteNewsRepository::open(&db_path)?;
        let items = repo.list_news()?;
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "Persisted news");
        Ok(())
    }

    #[test]
    fn sqlite_repository_creates_parent_dirs() -> Result<(), Box<dyn Error>> {
        let dir = tempfile::tempdir()?;
        let db_path = dir.path().join("nested").join("dir").join("test.db");

        let repo = SqliteNewsRepository::open(&db_path)?;
        let items = repo.list_news()?;
        assert!(items.is_empty());
        assert!(db_path.exists());
        Ok(())
    }

    #[test]
    fn mock_remote_repository_preserves_dedup_and_stable_order() -> Result<(), Box<dyn Error>> {
        let mut repository = MockRemoteNewsRepository::new("trendradar");
        repository.save_news_batch(&[
            NewsItem {
                title: "Rust release".to_owned(),
                source_id: "weibo".to_owned(),
                rank: 5,
            },
            NewsItem {
                title: "Rust release".to_owned(),
                source_id: "weibo".to_owned(),
                rank: 2,
            },
            NewsItem {
                title: "AI report".to_owned(),
                source_id: "zhihu".to_owned(),
                rank: 2,
            },
        ])?;

        let stored = repository.list_news()?;
        assert_eq!(stored.len(), 2);
        assert_eq!(stored[0].source_id, "weibo");
        assert_eq!(stored[0].rank, 2);
        assert_eq!(stored[1].source_id, "zhihu");
        Ok(())
    }

    #[test]
    fn mock_remote_repository_reports_save_failure() {
        let mut repository = MockRemoteNewsRepository::fail_on_save("trendradar");
        let error = repository
            .save_news(NewsItem {
                title: "broken".to_owned(),
                source_id: "weibo".to_owned(),
                rank: 1,
            })
            .expect_err("mock remote save should fail");

        assert!(
            error
                .to_string()
                .contains("mock remote storage save failed")
        );
    }

    #[test]
    fn mock_remote_repository_reports_list_failure() {
        let repository = MockRemoteNewsRepository::fail_on_list("trendradar");
        let error = repository
            .list_news()
            .expect_err("mock remote list should fail");

        assert!(
            error
                .to_string()
                .contains("mock remote storage list failed")
        );
    }
}
