//! 存储抽象骨架。

use rusqlite::{Connection, params};
use trendradar_domain::TrendRadarError;
use trendradar_domain::{NewsItem, Result};

/// 新闻存储接口。
pub trait NewsRepository {
    /// 保存一条新闻。
    fn save_news(&mut self, item: NewsItem) -> Result<()>;

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

#[cfg(test)]
mod tests {
    use super::{NewsRepository, SqliteNewsRepository};
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
}
