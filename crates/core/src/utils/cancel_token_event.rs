use std::marker::PhantomData;

use tauri::{EventId, Listener, Runtime};
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub struct CancellationTokenListener<L, R>
where
    L: Listener<R>,
    R: Runtime,
{
    listener: L,
    event_id: EventId,
    cancel_token: CancellationToken,
    runtime: PhantomData<R>,
}

impl<L, R> Drop for CancellationTokenListener<L, R>
where
    L: Listener<R>,
    R: Runtime,
{
    fn drop(&mut self) {
        self.listener.unlisten(self.event_id);
    }
}

impl<L, R> CancellationTokenListener<L, R>
where
    L: Listener<R>,
    R: Runtime,
{
    pub fn new<E: Into<String>>(listener: L, event: E) -> Self {
        let token = CancellationToken::new();
        let cloned_token = token.clone();
        let event_id = listener.once(event, move |_| {
            cloned_token.cancel();
        });
        Self {
            listener,
            event_id,
            cancel_token: token,
            runtime: PhantomData::<R>,
        }
    }
    pub fn listener(&self) -> &L {
        &self.listener
    }
    pub fn token(&self) -> CancellationToken {
        self.cancel_token.child_token()
    }
}
