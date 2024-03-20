use crate::config::Config;
use crate::matrix::MatrixClient;
use ruma::{
    events::{
        room::message::{
            MessageEventContent, MessageType, RelatesTo, Relation, TextMessageEventContent,
        },
        AnyMessageEventContent,
    },
    identifiers::{EventId, RoomId},
    serde::Raw,
    OwnedUserId,
};
use serde_json::Value;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub struct Bot {
    matrix_client: MatrixClient,
}

impl Bot {
    pub async fn new(config: &Config) -> Result<Self, Box<dyn std::error::Error>> {
        let matrix_client = MatrixClient::new(config).await?;
        Ok(Self { matrix_client })
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            let events = self.matrix_client.sync().await?;
            for event in events {
                let event_processor = self.process_message_event(event.clone());
                tokio::spawn(async move {
                    if let Err(e) = event_processor.await {
                        eprintln!("Failed to process event: {}", e);
                    }
                });
            }
        }
    }

    async fn process_message_event(
        &self,
        event: &AnySyncRoomEvent,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let AnySyncRoomEvent::RoomMessage(msg_event) = event {
            if let Some(body) = msg_event.content().msgtype.text() {
                if body.starts_with("!ping") {
                    self.handle_ping_command(&msg_event.room_id, &body[5..].trim())
                        .await?;
                }
            }
        }
        Ok(())
    }

    async fn handle_ping_command(
        &self,
        room_id: &RoomId,
        sender: &UserId,
        event_id: &EventId,
        optional_text: &str,
        origin_server_ts: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let now = UNIX_EPOCH
            + Duration::from_secs(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            );
        let diff = now.as_millis() as u64 - origin_server_ts;

        // Format the optional text
        let message = if !optional_text.is_empty() {
            format!("\"{}\" ", optional_text)
        } else {
            "".to_string()
        };

        let plain_body = format!(
            "{}: Pong! (ping {}took {} ms to arrive)",
            sender, message, diff
        );

        let formatted_body = format!(
            "<a href='https://matrix.to/#/{sender}'>{sender}</a>: Pong! \
            (<a href='https://matrix.to/#/{room_id}/{event_id}'>ping</a> {message}took {diff} ms to arrive)"
        );

        let content = MessageEventContent::notice_formatted(&plain_body, &formatted_body);

        self.matrix_client.send_message(room_id, content).await?;
        Ok(())
    }
}
