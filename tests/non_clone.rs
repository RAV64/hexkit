use hexkit::{Handle, HandleMut};

struct RegisterUser {
    username: String,
}

struct UserId(u64);

struct GetUser<'i> {
    id: &'i UserId,
}

struct UserStore {
    next_id: u64,
    users: Vec<(u64, String)>,
}

impl UserStore {
    fn new() -> Self {
        Self {
            next_id: 0,
            users: Vec::new(),
        }
    }
}

impl HandleMut<RegisterUser> for UserStore {
    type Output<'a> = Result<UserId, ()>;

    fn handle_mut(&mut self, input: RegisterUser) -> Self::Output<'_> {
        self.next_id += 1;
        self.users.push((self.next_id, input.username));
        Ok(UserId(self.next_id))
    }
}

impl Handle<GetUser<'_>> for UserStore {
    type Output<'a> = Result<Option<&'a str>, ()>;

    fn handle(&self, input: GetUser<'_>) -> Self::Output<'_> {
        Ok(self
            .users
            .iter()
            .find(|(id, _)| *id == input.id.0)
            .map(|(_, name)| name.as_str()))
    }
}

#[test]
fn traits_work_with_non_clone_requests() {
    let mut store = UserStore::new();

    let id = store
        .handle_mut(RegisterUser {
            username: String::from("lea"),
        })
        .expect("register should succeed");

    let name = store
        .handle(GetUser { id: &id })
        .expect("lookup should succeed")
        .expect("user should exist");

    assert_eq!(name, "lea");
}
