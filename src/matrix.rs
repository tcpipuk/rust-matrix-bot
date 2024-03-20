use ruma::{
    api::client::{
        r0::{
            account::register::Request as RegisterRequest,
            message::send_message_event,
            sync::sync_events,
        },
        error::Error as RumaClientError,
    },
    events::{
        room::message::{MessageEventContent, MessageType, TextMessageEventContent},
        AnyMessageEventContent,
    },
    Client, DeviceId, UserId,
};
use serde_json::json;
use tokio::time::{delay_for, Duration};

use crate::config::Config;

pub struct MatrixClient {
    client: Client,
    next_batch: Option<String>,
}

impl MatrixClient {
    pub async fn new(config: &Config) -> Result<Self, Box<dyn std::error::Error>> {
        let homeserver_url = config.homeserver_url.parse()?;
        let client = Client::new(homeserver_url, None);

        Ok(Self {
            client,
            next_batch: None,
        })
    }

    pub async fn login(&mut self, config: &Config) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref token) = config.auth_token {
            self.client.set_access_token(token.clone());
        } else {
            let user_id = UserId::parse(&config.user_id)?;
            let request = RegisterRequest::new();
            let response = self
                .client
                .send(request, Some(DeviceId::try_from(config.device_id.as_deref().unwrap_or(""))?))
                .await?;

            self.client.set_access_token(response.access_token);
        }
        Ok(())
    }

    pub async fn send_message(
        &self,
        room_id: &str,
        message: &str,
    ) -> Result<(), RumaClientError> {
        let content = MessageEventContent::text_plain(message);
        let txn_id = format!("{}", uuid::Uuid::new_v4());
        self.client
            .send(
                send_message_event::Request::new(
                    room_id.try_into()?,
                    &txn_id,
                    &content,
                ),
                None,
            )
            .await?;
        Ok(())
    }

    pub async fn sync(&mut self) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
        let request = sync_events::Request::new().since(self.next_batch.as_deref());
        let response = self.client.send(request, None).await?;

        // Update the next_batch token for the next incremental sync
        self.next_batch = Some(response.next_batch.clone());

        let mut events = Vec::new();

        // Collect all events from joined rooms
        for (_room_id, room) in response.rooms.join {
            for event in room.timeline.events {
                // Directly push the raw event to the collection without filtering
                let event_value = serde_json::to_value(event)?;
                events.push(event_value);
            }
        }

        Ok(events)
    }
}
