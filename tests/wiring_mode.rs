use hexkit::{Handle, HandleMut};

struct Register {
    value: u64,
}

struct ReadCounter;

#[derive(Default)]
struct Counter {
    n: u64,
}

impl HandleMut<Register> for Counter {
    type Output<'a> = Result<u64, ()>;

    fn handle_mut(&mut self, input: Register) -> Self::Output<'_> {
        self.n += input.value;
        Ok(self.n)
    }
}

impl Handle<ReadCounter> for Counter {
    type Output<'a> = u64;

    fn handle(&self, _input: ReadCounter) -> Self::Output<'_> {
        self.n
    }
}

#[test]
fn handler_traits_preserve_boundaries() {
    let mut core = Counter::default();

    let next = core
        .handle_mut(Register { value: 3 })
        .expect("increment should succeed");
    let now = core.handle(ReadCounter);

    assert_eq!(next, 3);
    assert_eq!(now, 3);
}
