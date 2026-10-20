use std::sync::RwLock;

use chat_core::{DeviceAddedEvent, MessagesEvent, NotifyPrefs};

use crate::rt::AppHandle;

#[derive(Default)]
pub struct Notifier {
    prefs: RwLock<NotifyPrefs>,
}

impl Notifier {
    pub fn prefs(&self, update: Option<NotifyPrefs>) -> NotifyPrefs {
        if let (Some(update), Ok(mut prefs)) = (update, self.prefs.write()) {
            *prefs = update;
        }
        self.prefs.read().map(|prefs| *prefs).unwrap_or_default()
    }
}

pub fn on_messages(_app: &AppHandle, _event: &MessagesEvent) {}

pub fn on_device_added(_app: &AppHandle, _event: &DeviceAddedEvent) {}
