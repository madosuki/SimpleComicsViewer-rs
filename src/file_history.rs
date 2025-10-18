use rusqlite::{Connection, OpenFlags, Result};

#[derive(Debug)]
pub struct FileHistory {
    id: i32,
    pub path: String,
    unixtime: u64,
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
        self.conn.execute("create table if not exists open_file_history (id integer primary key autoincrement, path text not null unique, unixtime integer not null)",
                          ()).unwrap();
    }

    pub fn add_history(&self, file_path: String, unixtime: u64) {
        self.conn.execute("insert into open_file_history (path, unixtime) values(?1, ?2)", [file_path, unixtime.to_string()]).unwrap();
    }

    pub fn update_history(&self, file_path: &str, unixtime: u64) {
        self.conn.execute("update open_file_history set unixtime = ?1 where path = ?2", [unixtime.to_string(), file_path.to_owned()]).unwrap();
    }

    pub fn is_exists_file_path(&self, file_path: &str) -> bool {
        let mut stmt = self.conn.prepare("select id from open_file_history where path = ?").unwrap();
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

    pub fn get_history(&self) -> Vec<FileHistory> {
        let mut file_history_list: Vec<FileHistory> = vec!();

        let mut stmt = self.conn.prepare("select id, path, unixtime from open_file_history order by unixtime desc limit 10").unwrap();
        let stmt_iter = stmt.query_map([], |row| {
            Ok (FileHistory {
                id: row.get(0).unwrap(),
                path: row.get(1).unwrap(),
                unixtime: row.get(2).unwrap()
            })
        }).unwrap();
        for result in stmt_iter {
            let tmp = result.unwrap();
            file_history_list.push(tmp);
        }
        
        file_history_list
    }
}
