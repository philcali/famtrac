use aws_sdk_dynamodb::Client;
use lambda_runtime::Error;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::classify::{change_kind, ChangeKind, RecordChange};

/// A handler function signature. Receives the DDB client, table name,
/// and a shared reference to the classified RecordChange.
/// Uses `Arc<RecordChange>` so multiple handlers can process the same event.
pub type HandlerFn = Box<
    dyn Fn(
            Arc<Client>,
            String,
            Arc<RecordChange>,
        ) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send>>
        + Send
        + Sync,
>;

/// Dispatch table that maps `ChangeKind` discriminants to async handler functions.
/// Constructed once during Lambda cold-start and reused across invocations.
#[derive(Default)]
pub struct Router {
    handlers: HashMap<ChangeKind, Vec<HandlerFn>>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// Register a handler for the given `ChangeKind`. Multiple handlers can be
    /// registered for the same kind — they all execute on dispatch.
    pub fn register(&mut self, kind: ChangeKind, handler: HandlerFn) {
        self.handlers.entry(kind).or_default().push(handler);
    }

    /// Dispatch a classified `RecordChange` to all matching handlers.
    ///
    /// - `Ignored` variants are skipped (returns `Ok(())`).
    /// - All registered handlers for the variant's `ChangeKind` are invoked,
    ///   even if earlier handlers fail.
    /// - Returns the first error encountered, if any.
    pub async fn dispatch(
        &self,
        client: Arc<Client>,
        table_name: &str,
        change: RecordChange,
    ) -> Result<(), Error> {
        let kind = match change_kind(&change) {
            Some(k) => k,
            None => return Ok(()), // Ignored — skip
        };

        let handlers = match self.handlers.get(&kind) {
            Some(h) => h,
            None => return Ok(()),
        };

        let change = Arc::new(change);
        let mut errors: Vec<Error> = Vec::new();

        for handler in handlers {
            if let Err(e) = handler(
                Arc::clone(&client),
                table_name.to_string(),
                Arc::clone(&change),
            )
            .await
            {
                errors.push(e);
            }
        }

        if let Some(first) = errors.into_iter().next() {
            Err(first)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use famtrac_backend::domain::Share;

    #[test]
    fn test_router_new_is_empty() {
        let router = Router::new();
        assert!(router.handlers.is_empty());
    }

    /// Helper: build a minimal DDB client for testing (never actually called).
    fn test_client() -> Arc<Client> {
        let conf = aws_sdk_dynamodb::Config::builder()
            .behavior_version(aws_sdk_dynamodb::config::BehaviorVersion::latest())
            .build();
        Arc::new(Client::from_conf(conf))
    }

    /// Helper: build a minimal Share for testing.
    fn test_share() -> Share {
        use famtrac_backend::domain::*;
        Share {
            id: ShareId::new(),
            family_id: FamilyId::new(),
            requester_id: IdentityId::new("requester".to_string()),
            accepter_id: Some(IdentityId::new("accepter".to_string())),
            accepter_username: "accepter@example.com".to_string(),
            permission_scope: PermissionScope {
                actions: vec![PermissionAction::FamilyRead],
            },
            status: ShareStatus::Active,
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
            expires_at: None,
        }
    }

    #[tokio::test]
    async fn test_dispatch_skips_ignored() {
        let router = Router::new();
        let client = test_client();
        let result = router
            .dispatch(client, "table", RecordChange::Ignored)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_dispatch_no_handlers_registered() {
        let router = Router::new();
        let client = test_client();
        let change = RecordChange::ShareActivated(test_share());
        let result = router.dispatch(client, "table", change).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_dispatch_invokes_matching_handler() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);

        let mut router = Router::new();
        router.register(
            ChangeKind::ShareActivated,
            Box::new(move |_client, _table, _change| {
                let c = Arc::clone(&counter_clone);
                Box::pin(async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                })
            }),
        );

        let client = test_client();
        let change = RecordChange::ShareActivated(test_share());
        let result = router.dispatch(client, "table", change).await;
        assert!(result.is_ok());
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_dispatch_invokes_all_handlers_for_same_kind() {
        let counter = Arc::new(AtomicUsize::new(0));

        let mut router = Router::new();
        for _ in 0..3 {
            let c = Arc::clone(&counter);
            router.register(
                ChangeKind::ShareActivated,
                Box::new(move |_client, _table, _change| {
                    let c = Arc::clone(&c);
                    Box::pin(async move {
                        c.fetch_add(1, Ordering::SeqCst);
                        Ok(())
                    })
                }),
            );
        }

        let client = test_client();
        let change = RecordChange::ShareActivated(test_share());
        let result = router.dispatch(client, "table", change).await;
        assert!(result.is_ok());
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_dispatch_does_not_invoke_non_matching_handlers() {
        let activated_counter = Arc::new(AtomicUsize::new(0));
        let revoked_counter = Arc::new(AtomicUsize::new(0));

        let mut router = Router::new();

        let c = Arc::clone(&activated_counter);
        router.register(
            ChangeKind::ShareActivated,
            Box::new(move |_client, _table, _change| {
                let c = Arc::clone(&c);
                Box::pin(async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                })
            }),
        );

        let c = Arc::clone(&revoked_counter);
        router.register(
            ChangeKind::ShareRevoked,
            Box::new(move |_client, _table, _change| {
                let c = Arc::clone(&c);
                Box::pin(async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                })
            }),
        );

        let client = test_client();
        let change = RecordChange::ShareActivated(test_share());
        router.dispatch(client, "table", change).await.unwrap();

        assert_eq!(activated_counter.load(Ordering::SeqCst), 1);
        assert_eq!(revoked_counter.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn test_dispatch_error_isolation_all_handlers_run() {
        let counter = Arc::new(AtomicUsize::new(0));

        let mut router = Router::new();

        // Handler 1: fails
        let c = Arc::clone(&counter);
        router.register(
            ChangeKind::ShareActivated,
            Box::new(move |_client, _table, _change| {
                let c = Arc::clone(&c);
                Box::pin(async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Err("handler 1 failed".into())
                })
            }),
        );

        // Handler 2: succeeds
        let c = Arc::clone(&counter);
        router.register(
            ChangeKind::ShareActivated,
            Box::new(move |_client, _table, _change| {
                let c = Arc::clone(&c);
                Box::pin(async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                })
            }),
        );

        // Handler 3: fails
        let c = Arc::clone(&counter);
        router.register(
            ChangeKind::ShareActivated,
            Box::new(move |_client, _table, _change| {
                let c = Arc::clone(&c);
                Box::pin(async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Err("handler 3 failed".into())
                })
            }),
        );

        let client = test_client();
        let change = RecordChange::ShareActivated(test_share());
        let result = router.dispatch(client, "table", change).await;

        // All 3 handlers ran despite errors
        assert_eq!(counter.load(Ordering::SeqCst), 3);
        // Returns the first error
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("handler 1 failed"));
    }

    #[tokio::test]
    async fn test_register_new_handler_does_not_affect_existing() {
        let first_counter = Arc::new(AtomicUsize::new(0));
        let second_counter = Arc::new(AtomicUsize::new(0));

        let mut router = Router::new();

        let c = Arc::clone(&first_counter);
        router.register(
            ChangeKind::ResourceChanged,
            Box::new(move |_client, _table, _change| {
                let c = Arc::clone(&c);
                Box::pin(async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                })
            }),
        );

        // Register a second handler for the same kind
        let c = Arc::clone(&second_counter);
        router.register(
            ChangeKind::ResourceChanged,
            Box::new(move |_client, _table, _change| {
                let c = Arc::clone(&c);
                Box::pin(async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                })
            }),
        );

        let client = test_client();
        let change = RecordChange::ResourceChanged(crate::classify::ResourceChange {
            pk: "PK".to_string(),
            sk: "SK".to_string(),
            operation: crate::classify::ChangeOperation::Insert,
            new_image: HashMap::new(),
            old_image: HashMap::new(),
        });
        router.dispatch(client, "table", change).await.unwrap();

        // Both handlers invoked
        assert_eq!(first_counter.load(Ordering::SeqCst), 1);
        assert_eq!(second_counter.load(Ordering::SeqCst), 1);
    }
}
