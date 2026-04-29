use rusqlite::{Connection, OpenFlags, params};

#[derive(Debug)]
pub struct FileHistory {
    id: i32,
    pub location_path: String,
    unixtime: i64,
    last_show_page_index: i64,
}

pub struct DbManager {
    conn: Connection
}

unsafe impl Send for DbManager {}
unsafe impl Sync for DbManager {}

impl DbManager {
    pub fn new(sqlite_file_path: &str) -> Self {
        let conn = Connection::open_with_flags(sqlite_file_path, OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE).expect("failed open sqlite3 file");
        DbManager { conn: conn }
    }

    pub fn init(&self) {
        self.conn.execute("create table if not exists open_file_history (id integer primary key autoincrement, location_path text not null unique, unixtime integer not null, last_show_page_index integer not null)",
                          ()).unwrap();
    }

    pub fn add_history(&self, file_path: String, unixtime: i64) {
        self.conn.execute("insert into open_file_history (location_path, unixtime, last_show_page_index) values(?1, ?2, ?3)",
                          params![file_path, unixtime, 0]).unwrap();
    }

    pub fn update_history(&self, file_path: &str, unixtime: i64, page_index: i64) {
        self.conn.execute("update open_file_history set unixtime = ?1, last_show_page_index = ?2 where location_path = ?3",
                          params![unixtime, page_index, file_path.to_owned()]).unwrap();
    }

    pub fn update_page_index(&self, file_path: &str, page_index: i64) {
        self.conn.execute("update open_file_history set last_show_page_index = ?1 where location_path = ?2",
                          params![page_index, file_path.to_owned()]).unwrap();
        
    }

    pub fn is_exists_file_path(&self, file_path: &str) -> bool {
        let mut stmt = self.conn.prepare("select id from open_file_history where location_path = ?").unwrap();
        let stmt_iter = stmt.query_map([file_path], |row| {
            let id: i32 = row.get(0).unwrap();
            Ok(id)
        }).unwrap();
        if stmt_iter.count() != 0 {
            true
        } else {
            false
        }
    }

    pub fn get_last_page_index(&self, file_path: &str) -> Option<i64> {
        let mut stmt = self.conn.prepare("select last_show_page_index from open_file_history where location_path = ?").unwrap();
        let mut stmt_iter = stmt.query_map([file_path], |row| {
            let last_show_page_index: i64 = row.get(3).unwrap();
            Ok(last_show_page_index)
        }).unwrap();

        if let Some(last_show_page_index) = stmt_iter.next() {
            match last_show_page_index {
                Ok(v) => {
                    return Some(v);
                },
                Err(e) => {
                    eprintln!("{e}");
                    return None;
                }
            }
        };
        None
    }

    pub fn get_history(&self) -> Vec<FileHistory> {
        let mut file_history_list: Vec<FileHistory> = vec!();

        let mut stmt = self.conn.prepare("select id, location_path, unixtime, last_show_page_index from open_file_history order by unixtime desc limit 10").unwrap();
        let stmt_iter = stmt.query_map([], |row| {
            let unixtime: i64 = row.get(2).unwrap();
            Ok (FileHistory {
                id: row.get(0).unwrap(),
                location_path: row.get(1).unwrap(),
                unixtime: row.get(2).unwrap(),
                last_show_page_index: row.get(3).unwrap()
            })
        }).unwrap();
        for result in stmt_iter {
            let tmp = result.unwrap();
            file_history_list.push(tmp);
        }
        
        file_history_list
    }
}
