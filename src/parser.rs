use std::process::Stdio;

use tokio::process::Command;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc::{channel, Receiver};

#[derive(Debug)]
pub enum MessageEvent
{
	Message
	{
		level: String,
		rendered: String,
	},
	Finished,
	Failed(anyhow::Error),
}

pub async fn spawn_cargo_build() -> anyhow::Result<Receiver<MessageEvent>>
{
	let (tx, rx) = channel(128);

	let mut child = Command::new("cargo")
		.args(&["build", "--message-format=json"])
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.spawn()?;

	let stdout = child
		.stdout
		.take()
		.ok_or(anyhow::anyhow!("Unable to obtain stdout"))?;

	let mut reader = BufReader::new(stdout).lines();

	tokio::spawn(async move {
		while let Ok(Some(line)) = reader.next_line().await
		{
			let Ok(json_data) = serde_json::from_str::<serde_json::Value>(&line)
			else
			{
				continue;
			};

			if json_data.get("reason").and_then(|r| r.as_str()) != Some("compiler-message")
			{
				continue;
			}

			let Some(message) = json_data.get("message")
			else
			{
				continue;
			};

			let level = message
				.get("level")
				.and_then(|l| l.as_str())
				.unwrap_or("error");

			if let Some(rendered) = message.get("rendered").and_then(|r| r.as_str())
			{
				if tx
					.send(MessageEvent::Message {
						level: level.to_string(),
						rendered: rendered.to_string(),
					})
					.await
					.is_err()
				{
					return;
				}
			}
		}

		// Wait for cargo to exit
		match child.wait().await
		{
			Ok(_) =>
			{
				let _ = tx.send(MessageEvent::Finished).await;
			}
			Err(e) =>
			{
				let _ = tx.send(MessageEvent::Failed(e.into())).await;
			}
		}
	});

	Ok(rx)
}

pub fn filter_messages(
	messages: &Vec<(String, String)>,
	ignore_warnings: bool,
) -> Vec<(String, String)>
{
	if ignore_warnings
	{
		messages
			.iter()
			.filter(|(kind, _)| kind != "warning")
			.map(|message| message.clone())
			.collect()
	}
	else
	{
		messages.to_vec()
	}
}
