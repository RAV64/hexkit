//! Purpose: composition root wiring with trait-object driven adapters.

use hexkit::{Handle, HandleMut};

#[derive(Debug, PartialEq, Eq)]
struct InvoiceId(u64);

#[derive(Debug, PartialEq, Eq)]
struct Invoice {
    id: InvoiceId,
    customer: String,
    cents: u64,
}

struct CreateInvoice {
    customer: String,
    cents: u64,
}

struct GetInvoice<'i> {
    id: &'i InvoiceId,
}

#[derive(Debug, PartialEq, Eq)]
enum AppError {
    Invalid,
    Repo,
    Audit,
}

trait InvoiceRepo {
    fn create(&mut self, input: CreateInvoice) -> Result<InvoiceId, AppError>;
    fn get<'a>(&'a self, id: &InvoiceId) -> Option<&'a Invoice>;
}

trait AuditSink {
    fn record(&mut self, message: &str) -> Result<(), AppError>;
}

struct BillingCore {
    repo: Box<dyn InvoiceRepo>,
    audit: Box<dyn AuditSink>,
}

impl BillingCore {
    fn new(repo: Box<dyn InvoiceRepo>, audit: Box<dyn AuditSink>) -> Self {
        Self { repo, audit }
    }
}

impl HandleMut<CreateInvoice> for BillingCore {
    type Output<'a> = Result<InvoiceId, AppError>;

    fn handle_mut(&mut self, input: CreateInvoice) -> Self::Output<'_> {
        if input.customer.trim().is_empty() || input.cents == 0 {
            return Err(AppError::Invalid);
        }

        let id = self.repo.create(input).map_err(|_| AppError::Repo)?;
        self.audit
            .record("invoice_created")
            .map_err(|_| AppError::Audit)?;

        Ok(id)
    }
}

impl Handle<GetInvoice<'_>> for BillingCore {
    type Output<'a> = Option<&'a Invoice>;

    fn handle(&self, input: GetInvoice<'_>) -> Self::Output<'_> {
        self.repo.get(input.id)
    }
}

struct HttpCreateInvoice {
    customer_name: String,
    amount_cents: u64,
}

struct HttpGetInvoice<'i> {
    id: &'i InvoiceId,
}

struct App {
    core: BillingCore,
}

impl App {
    const fn new(core: BillingCore) -> Self {
        Self { core }
    }
}

impl HandleMut<HttpCreateInvoice> for App {
    type Output<'a> = Result<InvoiceId, AppError>;

    fn handle_mut(&mut self, input: HttpCreateInvoice) -> Self::Output<'_> {
        self.core.handle_mut(CreateInvoice {
            customer: input.customer_name,
            cents: input.amount_cents,
        })
    }
}

impl Handle<HttpGetInvoice<'_>> for App {
    type Output<'a> = Option<&'a Invoice>;

    fn handle(&self, input: HttpGetInvoice<'_>) -> Self::Output<'_> {
        self.core.handle(GetInvoice { id: input.id })
    }
}

#[derive(Default)]
struct InMemoryRepo {
    next_id: u64,
    rows: Vec<Invoice>,
}

impl InvoiceRepo for InMemoryRepo {
    fn create(&mut self, input: CreateInvoice) -> Result<InvoiceId, AppError> {
        self.next_id += 1;
        self.rows.push(Invoice {
            id: InvoiceId(self.next_id),
            customer: input.customer,
            cents: input.cents,
        });
        Ok(InvoiceId(self.next_id))
    }

    fn get<'a>(&'a self, id: &InvoiceId) -> Option<&'a Invoice> {
        self.rows.iter().find(|invoice| invoice.id.0 == id.0)
    }
}

#[derive(Default)]
struct InMemoryAudit {
    events: Vec<String>,
}

impl AuditSink for InMemoryAudit {
    fn record(&mut self, message: &str) -> Result<(), AppError> {
        self.events.push(String::from(message));
        Ok(())
    }
}

fn build_app() -> App {
    let repo: Box<dyn InvoiceRepo> = Box::new(InMemoryRepo::default());
    let audit: Box<dyn AuditSink> = Box::new(InMemoryAudit::default());
    let core = BillingCore::new(repo, audit);
    App::new(core)
}

fn main() {
    let mut app = build_app();

    let id = app
        .handle_mut(HttpCreateInvoice {
            customer_name: String::from("Acme"),
            amount_cents: 2500,
        })
        .expect("invoice create should succeed");

    let invoice = app
        .handle(HttpGetInvoice { id: &id })
        .expect("invoice should exist");

    println!(
        "sync-composition-root-trait-objects: id={} customer={} cents={}",
        invoice.id.0, invoice.customer, invoice.cents
    );
}
