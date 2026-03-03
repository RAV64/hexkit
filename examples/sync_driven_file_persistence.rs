//! Purpose: driven adapter backed by a simple file-based persistence store.

use hexkit::{Handle, HandleMut};
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Eq)]
struct UserId(u64);

struct CreateUser {
    username: String,
}

struct GetUser<'i> {
    id: &'i UserId,
}

#[derive(Debug, PartialEq, Eq)]
enum StoreError {
    InvalidInput,
    Duplicate,
    Io,
    Corrupt,
}

trait UserRepository {
    fn insert(&mut self, username: String) -> Result<UserId, StoreError>;
    fn get(&self, id: &UserId) -> Result<Option<String>, StoreError>;
}

struct UserCore<R> {
    repo: R,
}

impl<R> UserCore<R> {
    const fn new(repo: R) -> Self {
        Self { repo }
    }
}

impl<R> HandleMut<CreateUser> for UserCore<R>
where
    R: UserRepository + 'static,
{
    type Output<'a> = Result<UserId, StoreError>;

    fn handle_mut(&mut self, input: CreateUser) -> Self::Output<'_> {
        if input.username.trim().is_empty() {
            return Err(StoreError::InvalidInput);
        }
        self.repo.insert(input.username)
    }
}

impl<R> Handle<GetUser<'_>> for UserCore<R>
where
    R: UserRepository + 'static,
{
    type Output<'a> = Result<Option<String>, StoreError>;

    fn handle(&self, input: GetUser<'_>) -> Self::Output<'_> {
        self.repo.get(input.id)
    }
}

struct FileUserStore {
    path: PathBuf,
    rows: Vec<(u64, String)>,
    next_id: u64,
}

impl FileUserStore {
    fn open(path: impl Into<PathBuf>) -> Result<Self, StoreError> {
        let path = path.into();
        if !path.exists() {
            fs::write(&path, "").map_err(|_| StoreError::Io)?;
        }

        let raw = fs::read_to_string(&path).map_err(|_| StoreError::Io)?;
        let mut rows = Vec::new();
        let mut next_id = 0;

        for line in raw.lines() {
            if line.is_empty() {
                continue;
            }

            let (id_raw, username) = line.split_once('|').ok_or(StoreError::Corrupt)?;
            let id = id_raw.parse::<u64>().map_err(|_| StoreError::Corrupt)?;
            next_id = next_id.max(id);
            rows.push((id, String::from(username)));
        }

        Ok(Self {
            path,
            rows,
            next_id,
        })
    }

    fn flush(&self) -> Result<(), StoreError> {
        let mut data = String::new();
        for (id, username) in &self.rows {
            let _ = writeln!(data, "{id}|{username}");
        }
        fs::write(&self.path, data).map_err(|_| StoreError::Io)
    }

    fn path_for_demo() -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push("hexkit_persistence_example.txt");
        path
    }
}

impl UserRepository for FileUserStore {
    fn insert(&mut self, username: String) -> Result<UserId, StoreError> {
        if self.rows.iter().any(|(_, existing)| existing == &username) {
            return Err(StoreError::Duplicate);
        }

        self.next_id += 1;
        self.rows.push((self.next_id, username));
        self.flush()?;

        Ok(UserId(self.next_id))
    }

    fn get(&self, id: &UserId) -> Result<Option<String>, StoreError> {
        Ok(self
            .rows
            .iter()
            .find(|(row_id, _)| *row_id == id.0)
            .map(|(_, username)| username.clone()))
    }
}

fn cleanup(path: &Path) {
    let _ = fs::remove_file(path);
}

fn main() {
    let path = FileUserStore::path_for_demo();
    cleanup(&path);

    let store = FileUserStore::open(&path).expect("file store open should succeed");
    let mut app = UserCore::new(store);

    let lea = app
        .handle_mut(CreateUser {
            username: String::from("lea"),
        })
        .expect("create lea should succeed");
    let duplicate = app.handle_mut(CreateUser {
        username: String::from("lea"),
    });
    let lookup = app
        .handle(GetUser { id: &lea })
        .expect("lookup should succeed")
        .expect("lea should exist");

    assert_eq!(duplicate, Err(StoreError::Duplicate));
    assert_eq!(lookup, "lea");

    println!(
        "sync-driven-file-persistence: id={} username={lookup}",
        lea.0
    );

    cleanup(&path);
}
