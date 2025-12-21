mod cli;
mod highlighting;
mod parser;
mod ui;

use cli::*;
use parser::*;
use ui::*;

use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()>
{
	let Cli::Fixie(cli) = Cli::parse();

	let mut ignore_warnings = cli.ignore_warnings;

	let mut index = 0;

	let (mut terminal, syntax_set, theme_set) = setup_tui()?;
	let mut build_rx = spawn_cargo_build().await?;

	let mut messages = Vec::new();
	let mut filtered_messages = Vec::new();

	let mut warnings = 0;
	let mut errors = 0;
	let mut others = 0;

	let mut building_index = 0;
	let mut building = true;

	loop
	{
		while let Ok(event) = build_rx.try_recv()
		{
			match event
			{
				MessageEvent::Message { level, rendered } if building =>
				{
					match level.as_str()
					{
						"warning" => warnings += 1,
						"error" => errors += 1,
						_ => others += 1,
					}

					messages.push((level, rendered));
				}

				MessageEvent::Finished =>
				{
					filtered_messages = filter_messages(&messages, ignore_warnings);
					building = false;
					index = 0;
				}

				MessageEvent::Failed(err) =>
				{
					cleanup_tui()?;
					return Err(err);
				}

				_ =>
				{}
			}
		}

		if building
		{
			let spinner_char = SPINNER_FRAMES[building_index];

			render_plain_message(
				&mut terminal,
				format!("{} Waiting for cargo build...", spinner_char).to_string(),
				Some((warnings, errors, others)),
				ignore_warnings,
			)?;

			building_index = (building_index + 1) % SPINNER_FRAMES.len();
		}
		else if filtered_messages.is_empty()
		{
			render_plain_message(
				&mut terminal,
				"Build finished with no warnings or errors".to_string(),
				None,
				ignore_warnings,
			)?;
		}
		else
		{
			if index >= filtered_messages.len()
			{
				index = 0;
			}

			render_message(
				&mut terminal,
				&syntax_set,
				&theme_set,
				&cli.theme,
				&filtered_messages[index],
				index,
				filtered_messages.len(),
				warnings,
				errors,
				others,
				ignore_warnings,
			)?;
		}

		match control_tui(index, filtered_messages.len())
		{
			Ok(Command::SwitchError(new_index)) if !building =>
			{
				index = new_index;
			}

			Ok(Command::IgnoreWarnings) if !building =>
			{
				index = 0;
				ignore_warnings = !ignore_warnings;
				filtered_messages = filter_messages(&messages, ignore_warnings);
			}

			Ok(Command::Rebuild) =>
			{
				index = 0;
				messages.clear();
				filtered_messages.clear();
				warnings = 0;
				errors = 0;
				others = 0;
				building = true;
				building_index = 0;

				build_rx = spawn_cargo_build().await?;
			}

			Ok(Command::Quit) => break,
			_ =>
			{}
		}

		tokio::task::yield_now().await;
	}

	cleanup_tui()?;

	Ok(())
}
