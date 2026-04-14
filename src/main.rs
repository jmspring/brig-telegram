//! brig-telegram: Telegram Bot API gateway for Brig
//!
//! Bridges Telegram messages to Brig's unix domain socket.
//! No async, no bot framework — just synchronous HTTP via ureq
//! and blocking socket reads.

use serde::{Deserialize, Serialize};
use std::env;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::process;
use std::time::Duration;

const DEFAULT_SOCKET: &str = "/var/brig/sock/brig.sock";
const TELEGRAM_API: &str = "https://api.telegram.org";
const POLL_TIMEOUT: u64 = 30;

// --- Brig socket protocol types ---

#[derive(Serialize)]
struct BrigHello<'a> {
    #[serde(rename = "type")]
    msg_type: &'static str,
    name: &'a str,
    version: &'static str,
}

#[derive(Deserialize)]
struct BrigWelcome {
    #[serde(rename = "type")]
    msg_type: String,
    #[serde(default)]
    capabilities: Vec<String>,
}

#[derive(Serialize)]
struct BrigTask {
    #[serde(rename = "type")]
    msg_type: &'static str,
    content: String,
    session: String,
}

#[derive(Deserialize)]
struct BrigMessage {
    #[serde(rename = "type")]
    msg_type: String,
    #[serde(default)]
    content: String,
    #[serde(default)]
    skill: String,
    #[serde(default)]
    state: String,
    #[serde(default)]
    code: String,
    #[serde(default)]
    message: String,
}

// --- Telegram API types ---

#[derive(Deserialize)]
struct TelegramResponse<T> {
    ok: bool,
    result: Option<T>,
    description: Option<String>,
}

#[derive(Deserialize)]
struct TelegramUpdate {
    update_id: i64,
    message: Option<TelegramMessage>,
}

#[derive(Deserialize)]
struct TelegramMessage {
    #[allow(dead_code)]
    message_id: i64,
    chat: TelegramChat,
    from: Option<TelegramUser>,
    text: Option<String>,
}

#[derive(Deserialize)]
struct TelegramChat {
    id: i64,
}

#[derive(Deserialize)]
struct TelegramUser {
    id: i64,
    is_bot: bool,
}

#[derive(Serialize)]
struct SendMessage {
    chat_id: i64,
    text: String,
    parse_mode: Option<&'static str>,
}

// --- Brig socket connection ---

struct BrigConnection {
    reader: BufReader<UnixStream>,
    writer: UnixStream,
}

impl BrigConnection {
    fn connect(socket_path: &str, gateway_name: &str) -> Result<Self, String> {
        let stream = UnixStream::connect(socket_path)
            .map_err(|e| format!("failed to connect to brig socket at {}: {}", socket_path, e))?;

        let writer = stream.try_clone()
            .map_err(|e| format!("failed to clone socket: {}", e))?;
        let reader = BufReader::new(stream);

        let mut conn = BrigConnection { reader, writer };
        conn.handshake(gateway_name)?;
        Ok(conn)
    }

    fn handshake(&mut self, gateway_name: &str) -> Result<(), String> {
        let hello = BrigHello {
            msg_type: "hello",
            name: gateway_name,
            version: "0.1.0",
        };
        self.send(&hello)?;

        let welcome: BrigWelcome = self.recv()?;
        if welcome.msg_type != "welcome" {
            return Err(format!("expected welcome, got {}", welcome.msg_type));
        }

        if !welcome.capabilities.contains(&"submit_task".to_string()) {
            return Err("brig does not grant submit_task capability".to_string());
        }

        eprintln!("connected to brig, capabilities: {:?}", welcome.capabilities);
        Ok(())
    }

    fn send<T: Serialize>(&mut self, msg: &T) -> Result<(), String> {
        let json = serde_json::to_string(msg)
            .map_err(|e| format!("failed to serialize message: {}", e))?;
        writeln!(self.writer, "{}", json)
            .map_err(|e| format!("failed to write to socket: {}", e))?;
        self.writer.flush()
            .map_err(|e| format!("failed to flush socket: {}", e))?;
        Ok(())
    }

    fn recv<T: for<'de> Deserialize<'de>>(&mut self) -> Result<T, String> {
        let mut line = String::new();
        self.reader.read_line(&mut line)
            .map_err(|e| format!("failed to read from socket: {}", e))?;
        if line.is_empty() {
            return Err("socket closed".to_string());
        }
        serde_json::from_str(&line)
            .map_err(|e| format!("failed to parse message: {} (line: {})", e, line.trim()))
    }

    fn submit_task(&mut self, content: &str, session: &str) -> Result<String, String> {
        let task = BrigTask {
            msg_type: "task",
            content: content.to_string(),
            session: session.to_string(),
        };
        self.send(&task)?;

        // Read status lines until we get a response
        loop {
            let msg: BrigMessage = self.recv()?;
            match msg.msg_type.as_str() {
                "response" => return Ok(msg.content),
                "status" => {
                    eprintln!("  [{}] {} - {}", msg.skill, msg.state, session);
                }
                "error" => {
                    return Err(format!("brig error {}: {}", msg.code, msg.message));
                }
                other => {
                    eprintln!("  unexpected message type: {}", other);
                }
            }
        }
    }
}

// --- Telegram API client ---

struct TelegramClient {
    token: String,
    agent: ureq::Agent,
}

impl TelegramClient {
    fn new(token: String) -> Self {
        let agent = ureq::AgentBuilder::new()
            .timeout_read(Duration::from_secs(POLL_TIMEOUT + 5))
            .timeout_write(Duration::from_secs(10))
            .build();
        TelegramClient { token, agent }
    }

    fn api_url(&self, method: &str) -> String {
        format!("{}/bot{}/{}", TELEGRAM_API, self.token, method)
    }

    fn get_updates(&self, offset: i64) -> Result<Vec<TelegramUpdate>, String> {
        let url = format!(
            "{}?offset={}&timeout={}",
            self.api_url("getUpdates"),
            offset,
            POLL_TIMEOUT
        );

        let response: TelegramResponse<Vec<TelegramUpdate>> = self.agent.get(&url)
            .call()
            .map_err(|e| format!("getUpdates failed: {}", e))?
            .into_json()
            .map_err(|e| format!("failed to parse getUpdates response: {}", e))?;

        if !response.ok {
            return Err(format!(
                "Telegram API error: {}",
                response.description.unwrap_or_default()
            ));
        }

        Ok(response.result.unwrap_or_default())
    }

    fn send_message(&self, chat_id: i64, text: &str) -> Result<(), String> {
        let msg = SendMessage {
            chat_id,
            text: text.to_string(),
            parse_mode: None,
        };

        let response: TelegramResponse<serde_json::Value> = self.agent
            .post(&self.api_url("sendMessage"))
            .send_json(&msg)
            .map_err(|e| format!("sendMessage failed: {}", e))?
            .into_json()
            .map_err(|e| format!("failed to parse sendMessage response: {}", e))?;

        if !response.ok {
            return Err(format!(
                "sendMessage error: {}",
                response.description.unwrap_or_default()
            ));
        }

        Ok(())
    }
}

// --- Main loop ---

fn run() -> Result<(), String> {
    let token = env::var("BRIG_TELEGRAM_TOKEN")
        .map_err(|_| "BRIG_TELEGRAM_TOKEN environment variable not set")?;

    let socket_path = env::var("BRIG_SOCKET")
        .unwrap_or_else(|_| DEFAULT_SOCKET.to_string());

    let gateway_name = env::var("BRIG_GATEWAY_NAME")
        .unwrap_or_else(|_| "telegram-gateway".to_string());

    let session_prefix = env::var("BRIG_SESSION_PREFIX")
        .unwrap_or_else(|_| "tg".to_string());

    eprintln!("{} starting", gateway_name);
    eprintln!("  socket: {}", socket_path);
    eprintln!("  session prefix: {}", session_prefix);

    let telegram = TelegramClient::new(token);
    let mut brig = BrigConnection::connect(&socket_path, &gateway_name)?;
    let mut update_offset: i64 = 0;

    eprintln!("polling for updates...");

    loop {
        let updates = match telegram.get_updates(update_offset) {
            Ok(u) => u,
            Err(e) => {
                eprintln!("telegram error: {}", e);
                std::thread::sleep(Duration::from_secs(5));
                continue;
            }
        };

        for update in updates {
            update_offset = update.update_id + 1;

            let message = match update.message {
                Some(m) => m,
                None => continue,
            };

            // Skip messages from bots
            if let Some(ref user) = message.from {
                if user.is_bot {
                    continue;
                }
            }

            // Skip messages without text
            let text = match message.text {
                Some(t) => t,
                None => continue,
            };

            let user_id = message.from.map(|u| u.id).unwrap_or(0);
            let chat_id = message.chat.id;
            let session = format!("{}-{}-{}", session_prefix, chat_id, user_id);

            eprintln!("[{}] <- {}", session, text);

            // Submit to brig and get response
            let response = match brig.submit_task(&text, &session) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("brig error: {}", e);
                    // Try to reconnect
                    match BrigConnection::connect(&socket_path, &gateway_name) {
                        Ok(new_conn) => {
                            brig = new_conn;
                            match brig.submit_task(&text, &session) {
                                Ok(r) => r,
                                Err(e) => format!("Error: {}", e),
                            }
                        }
                        Err(e) => format!("Error: brig unavailable ({})", e),
                    }
                }
            };

            eprintln!("[{}] -> {} chars", session, response.len());

            // Send response back to Telegram
            if let Err(e) = telegram.send_message(chat_id, &response) {
                eprintln!("failed to send response: {}", e);
            }
        }
    }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("fatal: {}", e);
        process::exit(1);
    }
}
