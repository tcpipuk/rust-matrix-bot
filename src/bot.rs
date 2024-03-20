use crate::config::Config;
use crate::matrix::MatrixClient;
use ruma::events::{
    room::message::{
        FormattedBodyFormat, MessageEventContent, MessageType, NoticeMessageEventContent, RelatesTo, RoomMessageEventContent, TextMessageEventContent,
    },
    AnyMessageEventContent, AnyMessageEventContent,
};
use ruma::{EventId, RoomId};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Bot {
    matrix_client: MatrixClient,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PongRelation {
    #[serde(rename = "rel_type")]
    pub rel_type: String,
    #[serde(rename = "event_id")]
    pub event_id: EventId,
}

impl From<PongRelation> for RelatesTo {
    fn from(pong: PongRelation) -> Self {
        Self::new(pong.rel_type, pong.event_id)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NoticeMessageEventContentWithRelatesTo {
    pub msgtype: MessageType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: String,
    #[serde(rename = "msgtype")]
    pub formatted: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relates_to: Option<RelatesTo>,
}

impl From<NoticeMessageEventContentWithRelatesTo> for AnyMessageEventContent {
    fn from(content: NoticeMessageEventContentWithRelatesTo) -> Self {
        AnyMessageEventContent::RoomMessage(RoomMessageEventContent::Notice(content))
    }
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
        sender: &str,
        event_id: &EventId,
        optional_text: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u64;
        let diff = now - event_id.as_secs() * 1000;

        let message_text = if !optional_text.is_empty() {
            format!("\"{}\" ", optional_text)
        } else {
            "".to_string()
        };

        // Create the message content with relation
        let content = RoomMessageEventContent {
            NoticeMessageEventContent {
                body: format!(
                    "{}: Pong! (ping {}took {} ms to arrive)",
                    sender, message_text, diff
                ),
                formatted: Some(FormattedBody {
                    body: format!(
                        "<a href='https://matrix.to/#/{sender}'>{sender}</a>: Pong! \
                        (<a href='https://matrix.to/#/{room_id}/{event_id}'>ping</a> {message_text}took {diff} ms to arrive)",
                        sender = sender,
                        room_id = room_id,
                        event_id = event_id,
                        message_text = message_text,
                        diff = diff
                    ),
                    format: MessageType::Html,
                }),
        },
            relates_to: Some(RelatesTo::from(PongRelation {
                rel_type: "xyz.maubot.pong".to_string(),
                event_id: event_id.clone(),
            })),
            msgtype: MessageType::Notice,
            mentions: sender,
        };

        self.matrix_client.send_message(room_id, &content).await?;
        Ok(())
    }
}
