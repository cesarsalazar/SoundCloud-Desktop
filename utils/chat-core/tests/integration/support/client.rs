use std::sync::{Arc, Mutex};
use std::time::Duration;

use chat_core::api::wire::{DeviceView, RegisterDevice, SendAnswer, SendMessage};
use chat_core::api::{Api, Identity};
use chat_core::channel::Channel;
use chat_core::chat_wire::prost::Message as _;
use chat_core::envelope::{self, Content, Post};
use chat_core::outbox::{Outbox, OutboxItem};
use chat_core::store::Store;
use chat_core::sync::{SyncEngine, SyncOutcome};
use chat_core::{
    ChatResult, Clock, ConvId, ConversationSummary, CoreEvent, Events, HistoryQuery, Message,
    MessageBody, Millis, RichText, Tier, async_trait,
};
use uuid::Uuid;

use super::people::Person;
use super::transport::HttpTransport;
use super::{Stack, eventually};

pub struct WallClock;

#[async_trait]
impl Clock for WallClock {
    fn now_ms(&self) -> Millis {
        super::people::now_ms() as Millis
    }

    async fn sleep(&self, ms: u64) {
        tokio::time::sleep(Duration::from_millis(ms)).await;
    }
}

#[derive(Default)]
pub struct Recorder(Mutex<Vec<CoreEvent>>);

impl Events for Recorder {
    fn emit(&self, event: CoreEvent) {
        self.0.lock().unwrap().push(event);
    }
}

impl Recorder {
    pub fn gone(&self) -> Vec<ConvId> {
        self.0
            .lock()
            .unwrap()
            .iter()
            .filter_map(|event| match event {
                CoreEvent::Gone(gone) => Some(gone.conversations.clone()),
                _ => None,
            })
            .flatten()
            .collect()
    }
}

pub struct Client {
    pub person: Person,
    pub transport: Arc<HttpTransport>,
    pub api: Arc<Api>,
    pub engine: SyncEngine,
    pub outbox: Outbox,
    pub events: Arc<Recorder>,
}

impl Client {
    pub async fn up(stack: &Stack, person: &Person) -> Self {
        let client = Self::on(vec![(Tier::Direct, stack.origin.clone())], person);
        client.api.config().await.expect("chat config answers");
        client.register().await.expect("the device registers");
        client
            .engine
            .devices()
            .await
            .expect("the device list loads");
        client.sync().await.expect("the first sync settles");
        client
    }

    pub fn on(routes: Vec<(Tier, String)>, person: &Person) -> Self {
        let transport = Arc::new(HttpTransport::new(routes));
        let channel = Arc::new(Channel::new(Arc::new(WallClock)));
        let api = Arc::new(Api::new(channel, transport.clone()));
        api.set_identity(Some(Identity {
            session_id: person.session_id.to_string(),
            device_id: person.device_id,
            keys: person.keys.clone(),
        }));
        let master: [u8; 32] = chat_core::chat_wire::random_bytes().unwrap();
        let store = Arc::new(Store::open_in_memory(&master).unwrap());
        store.set_me(&person.user_id).unwrap();
        let events = Arc::new(Recorder::default());
        let engine = SyncEngine::new(api.clone(), store.clone(), events.clone());
        engine.set_user(Some(person.user_id.clone()));
        Self {
            person: person.clone(),
            transport,
            api,
            engine,
            outbox: Outbox::new(store),
            events,
        }
    }

    pub fn store(&self) -> &Arc<Store> {
        self.engine.store()
    }

    pub async fn register(&self) -> ChatResult<DeviceView> {
        self.api
            .register_device(&RegisterDevice {
                platform: "linux".into(),
                name: format!("{} it", self.person.name),
                app_version: "0.0.0".into(),
                caps: Vec::new(),
            })
            .await
    }

    pub async fn sync(&self) -> ChatResult<SyncOutcome> {
        let plan = self.engine.next_plan()?;
        let outcome = self.engine.step(plan).await?;
        if outcome == SyncOutcome::Reset {
            self.engine.reconcile().await?;
        }
        Ok(outcome)
    }

    pub async fn sync_until(&self, what: &str, done: impl Fn(&Self) -> bool) {
        eventually(what, || async {
            self.sync().await.expect("sync answers");
            done(self)
        })
        .await;
    }

    pub async fn send(&self, conversation_id: ConvId, text: &str) -> ChatResult<usize> {
        let content = Content::Post(Post {
            text: RichText {
                text: text.to_owned(),
                spans: Vec::new(),
            },
            mentions: Vec::new(),
            embeds: Vec::new(),
            reply_to: None,
            forwarded_from: None,
        });
        let now = self.api.channel().now_ms();
        self.outbox.push(OutboxItem {
            client_id: Uuid::new_v4(),
            conversation_id,
            class: content.class(),
            target_seq: content.target(),
            envelope: envelope::encode(&content)?.encode_to_vec(),
            created_at: now,
            attempts: 0,
            next_at: now,
        })?;
        self.engine.flush_outbox().await
    }

    pub async fn raw_post(&self, conversation_id: ConvId) -> ChatResult<SendAnswer> {
        let view = self.api.conversation(conversation_id).await?;
        let message = SendMessage {
            client_id: Uuid::new_v4(),
            class: "post".into(),
            epoch: view.key_epoch,
            target_seq: None,
            body: vec![7; 64],
        };
        self.api.send(conversation_id, &message).await
    }

    pub fn messages(&self, conversation_id: ConvId) -> Vec<Message> {
        let query = HistoryQuery {
            before: None,
            after: None,
            limit: 200,
        };
        self.store()
            .messages(conversation_id, &query)
            .map(|page| page.messages)
            .unwrap_or_default()
    }

    pub fn texts(&self, conversation_id: ConvId) -> Vec<String> {
        self.messages(conversation_id)
            .into_iter()
            .filter_map(|message| match message.body {
                MessageBody::Text(rich) => Some(rich.text),
                MessageBody::Deleted => Some("<deleted>".into()),
                MessageBody::Locked => Some("<locked>".into()),
                MessageBody::System(_) => None,
                MessageBody::Unknown => Some("<unknown>".into()),
            })
            .collect()
    }

    pub fn conversation(&self, conversation_id: ConvId) -> Option<ConversationSummary> {
        self.store().conversation(conversation_id).ok().flatten()
    }
}
