use std::io::{BufRead, BufReader};
use std::process::{Stdio, Child, ChildStdout, Command};

pub fn spawn_cargo_build() -> anyhow::Result<(Child, BufReader<ChildStdout>)>
{
	let mut child = Command::new("cargo")
		.args(&["build", "--message-format=json"])
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.spawn()?;

	let stdout = child
		.stdout
		.take()
		.ok_or(anyhow::anyhow!("Unable to obtain stdout"))?;
	let reader = BufReader::new(stdout);

	Ok((child, reader))
}

pub fn parse_build_output(
	reader: BufReader<ChildStdout>,
	// ignore_warnings: bool,
) -> anyhow::Result<Vec<(String, String)>>
{
	let mut messages: Vec<(String, String)> = Vec::new();

	for line in reader.lines()
	{
		let line = line?;
		if let Ok(msg) = serde_json::from_str::<serde_json::Value>(&line)
		{
			if msg.get("reason").and_then(|r| r.as_str()) != Some("compiler-message")
			{
				continue;
			}

			if let Some(message) = msg.get("message")
			{
				let level = message
					.get("level")
					.and_then(|l| l.as_str())
					.unwrap_or("error"); // default to error

				// skip warnings
				// if ignore_warnings && level == "warning"
				// {
				// 	continue;
				// }

				if let Some(rendered) = message.get("rendered").and_then(|r| r.as_str())
				{
					messages.push((level.to_string(), rendered.to_string()));
				}
			}
		}
	}

	Ok(messages)
}

pub fn check_messages(messages: &Vec<(String, String)>) -> (usize, usize, usize)
{
	let warnings = messages
		.iter()
		.filter(|(level, _)| level == "warning")
		.count();
	let errors = messages
		.iter()
		.filter(|(level, _)| level == "error")
		.count();
	let others = messages.len() - warnings - errors;

	(warnings, errors, others)
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
