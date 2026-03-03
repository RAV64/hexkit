//! Purpose: smallest sync in-memory app with one write and one read interaction.

use hexkit::{Handle, HandleMut};

struct RegisterUser {
    username: String,
}

#[derive(Debug, PartialEq, Eq)]
struct UserId(u64);

struct GetUser<'i> {
    id: &'i UserId,
}

struct User {
    id: UserId,
    username: String,
}

#[derive(Default)]
struct UserStore {
    next_id: u64,
    users: Vec<User>,
}

impl HandleMut<RegisterUser> for UserStore {
    type Output<'a> = UserId;

    fn handle_mut(&mut self, input: RegisterUser) -> Self::Output<'_> {
        self.next_id += 1;
        self.users.push(User {
            id: UserId(self.next_id),
            username: input.username,
        });
        UserId(self.next_id)
    }
}

impl Handle<GetUser<'_>> for UserStore {
    type Output<'a> = Option<&'a str>;

    fn handle(&self, input: GetUser<'_>) -> Self::Output<'_> {
        self.users
            .iter()
            .find(|user| user.id.0 == input.id.0)
            .map(|user| user.username.as_str())
    }
}

fn main() {
    let mut app = UserStore::default();

    let id = app.handle_mut(RegisterUser {
        username: String::from("lea"),
    });
    let name = app
        .handle(GetUser { id: &id })
        .expect("created user should be present");

    println!("sync-basic-in-memory: id={} username={name}", id.0);
}
