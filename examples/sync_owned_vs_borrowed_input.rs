//! Purpose: compare owned vs borrowed driving inputs with the same core behavior.

use hexkit::{Handle, HandleMut};

#[derive(Debug, PartialEq, Eq)]
struct UserId(u64);

struct CreateUser {
    name: String,
}

struct ReadUser<'i> {
    id: &'i UserId,
}

#[derive(Default)]
struct UserCore {
    next_id: u64,
    rows: Vec<(u64, String)>,
}

impl HandleMut<CreateUser> for UserCore {
    type Output<'a> = UserId;

    fn handle_mut(&mut self, input: CreateUser) -> Self::Output<'_> {
        self.next_id += 1;
        self.rows.push((self.next_id, input.name));
        UserId(self.next_id)
    }
}

impl Handle<ReadUser<'_>> for UserCore {
    type Output<'a> = Option<&'a str>;

    fn handle(&self, input: ReadUser<'_>) -> Self::Output<'_> {
        self.rows
            .iter()
            .find(|(id, _)| *id == input.id.0)
            .map(|(_, name)| name.as_str())
    }
}

struct HttpCreateUser {
    display_name: String,
}

struct GrpcCreateUser<'i> {
    display_name: &'i str,
}

struct App {
    core: UserCore,
}

impl App {
    const fn new(core: UserCore) -> Self {
        Self { core }
    }
}

impl HandleMut<HttpCreateUser> for App {
    type Output<'a> = UserId;

    fn handle_mut(&mut self, input: HttpCreateUser) -> Self::Output<'_> {
        self.core.handle_mut(CreateUser {
            name: input.display_name,
        })
    }
}

impl HandleMut<GrpcCreateUser<'_>> for App {
    type Output<'a> = UserId;

    fn handle_mut(&mut self, input: GrpcCreateUser<'_>) -> Self::Output<'_> {
        self.core.handle_mut(CreateUser {
            name: input.display_name.to_owned(),
        })
    }
}

impl Handle<ReadUser<'_>> for App {
    type Output<'a> = Option<&'a str>;

    fn handle(&self, input: ReadUser<'_>) -> Self::Output<'_> {
        self.core.handle(input)
    }
}

fn main() {
    let mut app = App::new(UserCore::default());

    // HTTP-style adapter receives owned transport payload.
    let id_created_via_owned_http_input = app.handle_mut(HttpCreateUser {
        display_name: String::from("Lea"),
    });

    // gRPC-style adapter receives borrowed transport payload.
    let borrowed_payload = String::from("Sam");
    let id_created_via_borrowed_grpc_input = app.handle_mut(GrpcCreateUser {
        display_name: &borrowed_payload,
    });

    // Both adapters map into the same core interaction and read path.
    let name_from_owned_http_input = app
        .handle(ReadUser {
            id: &id_created_via_owned_http_input,
        })
        .expect("owned-input user should exist");
    let name_from_borrowed_grpc_input = app
        .handle(ReadUser {
            id: &id_created_via_borrowed_grpc_input,
        })
        .expect("borrowed-input user should exist");

    println!(
        "sync-owned-vs-borrowed-input: {}={} {}={}",
        id_created_via_owned_http_input.0,
        name_from_owned_http_input,
        id_created_via_borrowed_grpc_input.0,
        name_from_borrowed_grpc_input
    );
}
