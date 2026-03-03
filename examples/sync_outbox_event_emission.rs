//! Purpose: command handling that persists state and emits outbound events (outbox style).

use hexkit::{Handle, HandleMut};

#[derive(Debug, PartialEq, Eq)]
struct OrderId(u64);

#[derive(Debug, PartialEq, Eq)]
struct Order {
    id: OrderId,
    item: String,
}

struct PlaceOrder {
    item: String,
}

struct ReadOrder<'i> {
    id: &'i OrderId,
}

struct ReadOutbox;

#[derive(Debug, PartialEq, Eq)]
enum Event {
    OrderPlaced { id: u64, item: String },
}

#[derive(Debug, PartialEq, Eq)]
enum OrderError {
    Invalid,
    Store,
    Event,
}

trait OrderStore {
    fn insert(&mut self, item: String) -> Result<OrderId, OrderError>;
    fn get<'a>(&'a self, id: &OrderId) -> Option<&'a Order>;
}

trait EventSink {
    fn emit(&mut self, event: Event) -> Result<(), OrderError>;
    fn events(&self) -> &[Event];
}

struct OrderCore<S, E> {
    store: S,
    events: E,
}

impl<S, E> OrderCore<S, E> {
    const fn new(store: S, events: E) -> Self {
        Self { store, events }
    }
}

impl<S, E> HandleMut<PlaceOrder> for OrderCore<S, E>
where
    S: OrderStore + 'static,
    E: EventSink + 'static,
{
    type Output<'a> = Result<OrderId, OrderError>;

    fn handle_mut(&mut self, input: PlaceOrder) -> Self::Output<'_> {
        if input.item.trim().is_empty() {
            return Err(OrderError::Invalid);
        }

        let item_for_event = input.item.clone();
        let id = self
            .store
            .insert(input.item)
            .map_err(|_| OrderError::Store)?;
        self.events
            .emit(Event::OrderPlaced {
                id: id.0,
                item: item_for_event,
            })
            .map_err(|_| OrderError::Event)?;

        Ok(id)
    }
}

impl<S, E> Handle<ReadOrder<'_>> for OrderCore<S, E>
where
    S: OrderStore + 'static,
    E: EventSink + 'static,
{
    type Output<'a> = Option<&'a Order>;

    fn handle(&self, input: ReadOrder<'_>) -> Self::Output<'_> {
        self.store.get(input.id)
    }
}

impl<S, E> Handle<ReadOutbox> for OrderCore<S, E>
where
    S: OrderStore + 'static,
    E: EventSink + 'static,
{
    type Output<'a> = &'a [Event];

    fn handle(&self, _input: ReadOutbox) -> Self::Output<'_> {
        self.events.events()
    }
}

#[derive(Default)]
struct InMemoryOrderStore {
    next_id: u64,
    rows: Vec<Order>,
}

impl OrderStore for InMemoryOrderStore {
    fn insert(&mut self, item: String) -> Result<OrderId, OrderError> {
        self.next_id += 1;
        self.rows.push(Order {
            id: OrderId(self.next_id),
            item,
        });
        Ok(OrderId(self.next_id))
    }

    fn get<'a>(&'a self, id: &OrderId) -> Option<&'a Order> {
        self.rows.iter().find(|row| row.id.0 == id.0)
    }
}

#[derive(Default)]
struct InMemoryOutbox {
    rows: Vec<Event>,
}

impl EventSink for InMemoryOutbox {
    fn emit(&mut self, event: Event) -> Result<(), OrderError> {
        self.rows.push(event);
        Ok(())
    }

    fn events(&self) -> &[Event] {
        &self.rows
    }
}

fn main() {
    let mut app = OrderCore::new(InMemoryOrderStore::default(), InMemoryOutbox::default());

    let id = app
        .handle_mut(PlaceOrder {
            item: String::from("keyboard"),
        })
        .expect("order should succeed");

    let order = app
        .handle(ReadOrder { id: &id })
        .expect("order should exist");
    let events = app.handle(ReadOutbox);

    assert_eq!(events.len(), 1);
    println!(
        "sync-outbox-event-emission: id={} item={} events={}",
        order.id.0,
        order.item,
        events.len()
    );
}
