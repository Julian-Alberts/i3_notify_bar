use std::sync::{Arc, Mutex, PoisonError};

use crate::{notify_server_free_desktop::NotifyServerFreeDesktop, Event, NotificationId};

#[allow(async_fn_in_trait)]
#[mockall::automock]
pub trait NotifyServerInterface {
    async fn action_invoked(&self, id: NotificationId, action: &str) -> zbus::Result<()>;

    async fn notification_closed(
        &self,
        id: NotificationId,
        reason: CloseReason,
    ) -> zbus::Result<()>;
}

pub struct NotifyServer<Interface: NotifyServerInterface = NotifyServerFreeDesktop> {
    interface_ref: Interface,
    events: Arc<Mutex<Vec<Event>>>,
}

#[mockall::automock]
#[allow(async_fn_in_trait)]
pub trait NotificationSource {
    fn take_events(&mut self) -> Option<Vec<Event>>;
    async fn action_invoked(&self, id: NotificationId, action: &str) -> zbus::Result<()>;
    async fn notification_closed(
        &mut self,
        id: NotificationId,
        reason: &CloseReason,
    ) -> zbus::Result<()>;
}

impl NotifyServer<NotifyServerFreeDesktop> {
    // Calling this method in tests could break the notification service of operating systems.
    #[cfg(not(test))]
    pub fn start() -> zbus::Result<Self> {
        let events = Default::default();
        Ok(Self {
            interface_ref: NotifyServerFreeDesktop::new(Arc::clone(&events))?,
            events,
        })
    }
}

impl<I: NotifyServerInterface> NotificationSource for NotifyServer<I> {
    fn take_events(&mut self) -> Option<Vec<Event>> {
        let mut events = self.events.lock().unwrap_or_else(PoisonError::into_inner);
        if !events.is_empty() {
            Some(std::mem::take(&mut events))
        } else {
            None
        }
    }

    async fn action_invoked(&self, id: NotificationId, action: &str) -> zbus::Result<()> {
        self.interface_ref.action_invoked(id, action).await
    }

    async fn notification_closed(
        &mut self,
        id: NotificationId,
        reason: &CloseReason,
    ) -> zbus::Result<()> {
        self.events
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(Event::Close(id, *reason));
        self.interface_ref.notification_closed(id, *reason).await
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CloseReason {
    Expired = 1,
    Dismissed = 2,
    Closed = 3,
    Undefined = 4,
}

pub enum Message {
    NotificationClosed(u32, CloseReason),
    ActionInvoked(u32, String),
}

#[cfg(test)]
mod tests {
    use crate::{
        notify_server::NotifyServerInterface, CloseReason, Event, NotificationSource, NotifyServer,
    };

    #[async_std::test]
    async fn close_notification_event_is_added() {
        struct NotifyInterface;

        impl NotifyServerInterface for NotifyInterface {
            async fn action_invoked(&self, _: crate::NotificationId, _: &str) -> zbus::Result<()> {
                Ok(())
            }
            async fn notification_closed(
                &self,
                _: crate::NotificationId,
                _: crate::CloseReason,
            ) -> zbus::Result<()> {
                Ok(())
            }
        }

        let mut notify_server = NotifyServer {
            events: Default::default(),
            interface_ref: NotifyInterface,
        };

        notify_server
            .notification_closed(24.into(), &CloseReason::Dismissed)
            .await
            .unwrap();
        let events = notify_server.events.lock().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], Event::Close(24.into(), CloseReason::Dismissed));
    }
}
