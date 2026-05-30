use crate::types;

use rusqlite::{Connection, OpenFlags, params};
use types::PageDirection;

#[derive(Debug)]
#[allow(dead_code)]
pub struct FileHistory {
    id: i32,
    pub location_path: String,
    unixtime: i64,
    last_show_page_index: i64,
    pub page_direction: PageDirection,
}

pub struct DbManager {
    conn: Connection,
}

unsafe impl Send for DbManager {}
unsafe impl Sync for DbManager {}

impl DbManager {
    pub fn new(sqlite_file_path: &str) -> Self {
        let conn = Connection::open_with_flags(
            sqlite_file_path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
        )
        .expect("failed open sqlite3 file");
        DbManager { conn: conn }
    }

    fn get_open_file_history_columns(&self) -> Vec<String> {
        let mut stmt = self
            .conn
            .prepare("PRAGMA table_info(open_file_history)")
            .unwrap();
        let mut rows = stmt.query([]).unwrap();
        let mut columns = Vec::new();

        while let Some(row) = rows.next().unwrap() {
            columns.push(row.get(1).unwrap());
        }

        columns
    }

    fn has_open_file_history_column(&self, column_name: &str) -> bool {
        self.get_open_file_history_columns()
            .iter()
            .any(|name| name == column_name)
    }

    fn migrate(&self) {
        let user_version: i64 = self
            .conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();

        if user_version < 2 {
            if self.has_open_file_history_column("path")
                && !self.has_open_file_history_column("location_path")
            {
                self.conn
                    .execute_batch(
                        "alter table open_file_history rename column path to location_path;",
                    )
                    .unwrap();
            }

            if !self.has_open_file_history_column("last_show_page_index") {
                self.conn.execute_batch("alter table open_file_history add column last_show_page_index integer not null default 0;").unwrap();
            }

            if !self.has_open_file_history_column("page_direction") {
                self.conn.execute_batch("alter table open_file_history add column page_direction integer not null default 0;").unwrap();
            }

            self.conn.execute("PRAGMA user_version = 2", ()).unwrap();
        }
    }

    pub fn init(&self) {
        self.conn.execute("create table if not exists open_file_history (id integer primary key autoincrement, location_path text not null unique, unixtime integer not null, last_show_page_index integer not null, page_direction integer not null default 0)",
                          ()).unwrap();
        self.migrate();
    }

    pub fn add_history(&self, file_path: String, unixtime: i64, page_direction: PageDirection) {
        let page_direction_int = page_direction as i64;
        self.conn.execute("insert into open_file_history (location_path, unixtime, last_show_page_index, page_direction) values(?1, ?2, ?3, ?4)",
                          params![file_path, unixtime, 0, page_direction_int]).unwrap();
    }

    pub fn update_history(
        &self,
        file_path: &str,
        unixtime: i64,
        page_index: i64,
        page_direction: PageDirection,
    ) {
        let page_direction_int = page_direction as i64;
        self.conn.execute("update open_file_history set unixtime = ?1, last_show_page_index = ?2, page_direction = ?3 where location_path = ?4",
                          params![unixtime, page_index, page_direction_int, file_path.to_owned()]).unwrap();
    }

    pub fn update_page_index(&self, file_path: &str, page_index: i64) {
        self.conn
            .execute(
                "update open_file_history set last_show_page_index = ?1 where location_path = ?2",
                params![page_index, file_path.to_owned()],
            )
            .unwrap();
    }

    pub fn update_page_direction(&self, file_path: &str, page_direction: PageDirection) {
        let page_direction_int = page_direction as i64;
        self.conn
            .execute(
                "update open_file_history set page_direction = ?1 where location_path = ?2",
                params![page_direction_int, file_path.to_owned()],
            )
            .unwrap();
    }

    pub fn is_exists_file_path(&self, file_path: &str) -> bool {
        let mut stmt = self
            .conn
            .prepare("select id from open_file_history where location_path = ?")
            .unwrap();
        let stmt_iter = stmt
            .query_map([file_path], |row| {
                let id: i32 = row.get(0).unwrap();
                Ok(id)
            })
            .unwrap();
        if stmt_iter.count() != 0 { true } else { false }
    }

    pub fn get_last_page_index(&self, file_path: &str) -> Option<i64> {
        let mut stmt = self
            .conn
            .prepare("select last_show_page_index from open_file_history where location_path = ?")
            .unwrap();
        let mut stmt_iter = stmt
            .query_map([file_path], |row| {
                let last_show_page_index: i64 = row.get(0).unwrap();
                Ok(last_show_page_index)
            })
            .unwrap();

        if let Some(last_show_page_index) = stmt_iter.next() {
            match last_show_page_index {
                Ok(v) => {
                    return Some(v);
                }
                Err(e) => {
                    eprintln!("{e}");
                    return None;
                }
            }
        };
        None
    }

    pub fn get_pages_info(&self, file_path: &str) -> Option<(i64, PageDirection)> {
        let mut stmt = self.conn.prepare("select last_show_page_index, page_direction from open_file_history where location_path = ?").unwrap();
        let mut stmt_iter = stmt
            .query_map([file_path], |row| {
                let last_show_page_index: i64 = row.get(0).unwrap();

                let page_direction: i64 = row.get(1).unwrap();

                Ok((last_show_page_index, page_direction))
            })
            .unwrap();

        if let Some(v) = stmt_iter.next() {
            match v {
                Ok((last_show_page_index, page_direction_i64)) => {
                    let page_direction = PageDirection::try_from(page_direction_i64).unwrap();
                    return Some((last_show_page_index, page_direction));
                }
                Err(e) => {
                    eprintln!("{e}");
                    return None;
                }
            }
        };
        None
    }

    pub fn get_history(&self) -> Vec<FileHistory> {
        let mut file_history_list: Vec<FileHistory> = vec![];

        let mut stmt = self.conn.prepare("select id, location_path, unixtime, last_show_page_index, page_direction from open_file_history order by unixtime desc limit 10").unwrap();
        let stmt_iter = stmt
            .query_map([], |row| {
                let unixtime: i64 = row.get(2).unwrap();
                let page_direction_tmp: i64 = row.get(4).unwrap();
                let page_direction = PageDirection::try_from(page_direction_tmp).unwrap();
                Ok(FileHistory {
                    id: row.get(0).unwrap(),
                    location_path: row.get(1).unwrap(),
                    unixtime: unixtime,
                    last_show_page_index: row.get(3).unwrap(),
                    page_direction: page_direction,
                })
            })
            .unwrap();
        for result in stmt_iter {
            let tmp = result.unwrap();
            file_history_list.push(tmp);
        }

        file_history_list
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_db_path(test_name: &str) -> String {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir()
            .join(format!("simple_comics_viewer_{test_name}_{nanos}.db"))
            .to_string_lossy()
            .to_string()
    }

    #[test]
    fn init_migrates_old_history_schema() {
        let db_path = make_test_db_path("old_history_schema");

        {
            let conn = Connection::open(&db_path).unwrap();
            conn.execute_batch(
                "create table open_file_history (
                    id integer primary key autoincrement,
                    path text not null unique,
                    unixtime integer not null,
                    page_direction integer not null default 0
                );
                insert into open_file_history (path, unixtime, page_direction)
                values ('/tmp/sample.cbz', 10, 0);
                PRAGMA user_version = 1;",
            )
            .unwrap();
        }

        let db = DbManager::new(&db_path);
        db.init();

        assert!(db.has_open_file_history_column("location_path"));
        assert!(db.has_open_file_history_column("last_show_page_index"));
        assert_eq!(db.get_history()[0].location_path, "/tmp/sample.cbz");
        assert_eq!(db.get_last_page_index("/tmp/sample.cbz"), Some(0));

        std::fs::remove_file(db_path).unwrap();
    }
}
