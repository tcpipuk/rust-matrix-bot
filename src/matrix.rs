use crate::config::Config;
use hyper_tls::HttpsConnector;
use ruma::{
    api::client::message::send_message_event::v3::Request as SendMessageRequest,
    api::client::sync::sync_events, events::room::message::RoomMessageEventContent, Client, RoomId,
};
use serde_json::Value;
use std::convert::TryFrom;

impl MatrixClient {
    pub async fn new(config: &Config) -> Result<Self, Box<dyn std::error::Error>> {
        let homeserver_url = config.homeserver_url.parse()?;
        let https = HttpsConnector::new();
        let client = Client::new(https, config.homeserver_url.parse()?);

        Ok(Self {
            client,
            next_batch: None,
        })
    }

    pub async fn login(&mut self, config: &Config) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref token) = config.auth_token {
            self.client = Client::builder()
                .homeserver_url(config.homeserver_url)
                .access_token(Some(token.clone()))
                .build()
                .await?;
        } else {
            self.client = Client::builder()
                .homeserver_url(config.homeserver_url)
                .build()
                .await?;

            let session = self
                .client
                .log_in(
                    &config.user_id,
                    &config.password.as_deref().unwrap(),
                    None,
                    None,
                )
                .await?;

            config.update_auth_details(session.device_id.to_string(), session.access_token);
        }
        Ok(())
    }

    pub async fn send_message(
        &self,
        room_id: &str,
        content: RoomMessageEventContent,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let room_id = RoomId::try_from(&room_id)?;
        let txn_id = uuid::Uuid::new_v4().to_string();

        self.client
            .send_request(SendMessageRequest::new(&room_id, &txn_id, &content))
            .await?;

        Ok(())
    }

    pub async fn sync(&mut self) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
        let request = sync_events::v3::Request::new().since(self.next_batch.as_deref());
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
