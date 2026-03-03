use hexkit::{Handle, HandleMut};

mod registration_flow {
    use super::HandleMut;

    #[derive(Debug, PartialEq, Eq)]
    struct Register {
        username: &'static str,
    }

    #[derive(Debug, PartialEq, Eq)]
    struct Registered {
        id: u64,
        username: &'static str,
    }

    struct AuditRecord<'i> {
        registered: &'i Registered,
    }

    #[derive(Debug, PartialEq, Eq)]
    struct HttpRequest {
        username: &'static str,
    }

    #[derive(Debug, PartialEq, Eq)]
    struct CliCommand {
        username: &'static str,
    }

    #[derive(Debug, PartialEq, Eq)]
    enum RegisterError {
        Store,
        Audit,
    }

    #[derive(Debug, Default)]
    struct MemoryAccountStore {
        next_id: u64,
        accounts: std::vec::Vec<Registered>,
    }

    impl HandleMut<Register> for MemoryAccountStore {
        type Output<'a> = Result<Registered, RegisterError>;

        fn handle_mut(&mut self, input: Register) -> Self::Output<'_> {
            self.next_id += 1;
            self.accounts.push(Registered {
                id: self.next_id,
                username: input.username,
            });
            Ok(Registered {
                id: self.next_id,
                username: input.username,
            })
        }
    }

    #[derive(Debug, Default)]
    struct AuditLog {
        events: std::vec::Vec<&'static str>,
    }

    impl<'i> HandleMut<AuditRecord<'i>> for AuditLog {
        type Output<'a> = Result<(), RegisterError>;

        fn handle_mut(&mut self, input: AuditRecord<'i>) -> Self::Output<'_> {
            let event = if input.registered.username == "lea" {
                "register:lea"
            } else {
                "register:other"
            };
            self.events.push(event);
            Ok(())
        }
    }

    struct RegisterCore {
        store: MemoryAccountStore,
        audit: AuditLog,
    }

    impl HandleMut<Register> for RegisterCore {
        type Output<'a> = Result<Registered, RegisterError>;

        fn handle_mut(&mut self, input: Register) -> Self::Output<'_> {
            let created = self
                .store
                .handle_mut(input)
                .map_err(|_| RegisterError::Store)?;
            self.audit
                .handle_mut(AuditRecord {
                    registered: &created,
                })
                .map_err(|_| RegisterError::Audit)?;
            Ok(created)
        }
    }

    struct HttpRegisterAdapter<'a> {
        core: &'a mut RegisterCore,
    }

    impl HandleMut<HttpRequest> for HttpRegisterAdapter<'_> {
        type Output<'a>
            = Result<Registered, RegisterError>
        where
            Self: 'a;

        fn handle_mut(&mut self, input: HttpRequest) -> Self::Output<'_> {
            self.core.handle_mut(Register {
                username: input.username,
            })
        }
    }

    struct CliRegisterAdapter<'a> {
        core: &'a mut RegisterCore,
    }

    impl HandleMut<CliCommand> for CliRegisterAdapter<'_> {
        type Output<'a>
            = Result<Registered, RegisterError>
        where
            Self: 'a;

        fn handle_mut(&mut self, input: CliCommand) -> Self::Output<'_> {
            self.core.handle_mut(Register {
                username: input.username,
            })
        }
    }

    #[test]
    fn http_and_cli_input_adapters_share_same_core_and_outputs() {
        let mut core = RegisterCore {
            store: MemoryAccountStore::default(),
            audit: AuditLog::default(),
        };

        let first = {
            let mut http = HttpRegisterAdapter { core: &mut core };
            http.handle_mut(HttpRequest { username: "lea" })
                .expect("http registration should succeed")
        };

        let second = {
            let mut cli = CliRegisterAdapter { core: &mut core };
            cli.handle_mut(CliCommand { username: "sam" })
                .expect("cli registration should succeed")
        };

        assert_eq!(first.id, 1);
        assert_eq!(second.id, 2);
        assert_eq!(core.store.accounts.len(), 2);
        assert_eq!(core.audit.events, vec!["register:lea", "register:other"]);
    }
}

mod lookup_flow {
    use super::{Handle, HandleMut};

    #[derive(Debug, PartialEq, Eq)]
    struct HttpLookupRequest {
        id: u64,
    }

    #[derive(Debug, PartialEq, Eq)]
    struct GrpcLookupRequest {
        id: u64,
    }

    #[derive(Debug, PartialEq, Eq)]
    enum LookupError {
        Index,
        Audit,
    }

    struct LookupById {
        id: u64,
    }

    struct IndexLookup<'i> {
        id: &'i u64,
    }

    struct UsernameIndex {
        rows: &'static [(u64, &'static str)],
    }

    impl<'i> Handle<IndexLookup<'i>> for UsernameIndex {
        type Output<'a> = Result<Option<&'static str>, LookupError>;

        fn handle(&self, input: IndexLookup<'i>) -> Self::Output<'_> {
            Ok(self
                .rows
                .iter()
                .find(|(id, _)| id == input.id)
                .map(|(_, username)| *username))
        }
    }

    struct LookupAuditEvent;

    #[derive(Default)]
    struct QueryAudit {
        calls: usize,
        events: std::vec::Vec<&'static str>,
    }

    impl HandleMut<LookupAuditEvent> for QueryAudit {
        type Output<'a> = Result<(), LookupError>;

        fn handle_mut(&mut self, _input: LookupAuditEvent) -> Self::Output<'_> {
            self.calls += 1;
            self.events.push("lookup");
            Ok(())
        }
    }

    struct LookupCore {
        index: UsernameIndex,
        audit: QueryAudit,
    }

    impl HandleMut<LookupById> for LookupCore {
        type Output<'a> = Result<Option<&'static str>, LookupError>;

        fn handle_mut(&mut self, input: LookupById) -> Self::Output<'_> {
            self.audit
                .handle_mut(LookupAuditEvent)
                .map_err(|_| LookupError::Audit)?;
            self.index
                .handle(IndexLookup { id: &input.id })
                .map_err(|_| LookupError::Index)
        }
    }

    struct HttpLookupAdapter<'a> {
        core: &'a mut LookupCore,
    }

    impl HandleMut<HttpLookupRequest> for HttpLookupAdapter<'_> {
        type Output<'a>
            = Result<Option<&'static str>, LookupError>
        where
            Self: 'a;

        fn handle_mut(&mut self, input: HttpLookupRequest) -> Self::Output<'_> {
            self.core.handle_mut(LookupById { id: input.id })
        }
    }

    struct GrpcLookupAdapter<'a> {
        core: &'a mut LookupCore,
    }

    impl HandleMut<GrpcLookupRequest> for GrpcLookupAdapter<'_> {
        type Output<'a>
            = Result<Option<&'static str>, LookupError>
        where
            Self: 'a;

        fn handle_mut(&mut self, input: GrpcLookupRequest) -> Self::Output<'_> {
            self.core.handle_mut(LookupById { id: input.id })
        }
    }

    #[test]
    fn http_and_grpc_inputs_hit_same_read_core_and_outputs() {
        let mut core = LookupCore {
            index: UsernameIndex {
                rows: &[(1, "lea"), (2, "sam")],
            },
            audit: QueryAudit::default(),
        };

        let from_http = {
            let mut http = HttpLookupAdapter { core: &mut core };
            http.handle_mut(HttpLookupRequest { id: 1 })
                .expect("http lookup should succeed")
        };

        let from_grpc = {
            let mut grpc = GrpcLookupAdapter { core: &mut core };
            grpc.handle_mut(GrpcLookupRequest { id: 2 })
                .expect("grpc lookup should succeed")
        };

        assert_eq!(from_http, Some("lea"));
        assert_eq!(from_grpc, Some("sam"));
        assert_eq!(core.audit.calls, 2);
        assert_eq!(core.audit.events, vec!["lookup", "lookup"]);
    }
}
