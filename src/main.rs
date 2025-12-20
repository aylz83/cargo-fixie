mod cli;
mod highlighting;
mod parser;
mod ui;

use cli::*;
use parser::*;
use ui::*;

use clap::Parser;

fn main() -> anyhow::Result<()>
{
	let Cli::Fixie(cli) = Cli::parse();

	let (mut child, reader) = spawn_cargo_build()?;

	let mut ignore_warnings = cli.ignore_warnings;

	let mut messages = parse_build_output(reader)?;
	let mut filtered_messages = filter_messages(&messages, ignore_warnings);

	let (mut warnings, mut errors, mut others) = check_messages(&messages);

	child.wait()?;

	if messages.is_empty()
	{
		return Ok(());
	}

	let mut index = 0;

	let (mut terminal, syntax_set, theme_set) = setup_tui()?;

	loop
	{
		let message = &filtered_messages[index];
		render_message(
			&mut terminal,
			&syntax_set,
			&theme_set,
			&cli.theme,
			message,
			index,
			filtered_messages.len(),
			warnings,
			errors,
			others,
			ignore_warnings,
		)?;

		match control_tui(index, filtered_messages.len())
		{
			Ok(Command::SwitchError(new_index)) => index = new_index,
			Ok(Command::Quit) => break,
			Ok(Command::IgnoreWarnings) =>
			{
				index = 0;
				ignore_warnings = !ignore_warnings;
				filtered_messages = filter_messages(&messages, ignore_warnings);

				if filtered_messages.is_empty()
				{
					break;
				}
			}
			Ok(Command::Rebuild) =>
			{
				index = 0;
				let (mut new_child, new_reader) = spawn_cargo_build()?;
				messages = parse_build_output(new_reader)?;
				filtered_messages = filter_messages(&messages, ignore_warnings);
				let (new_warnings, new_errors, new_others) = check_messages(&messages);

				new_child.wait()?;
				child = new_child;

				warnings = new_warnings;
				errors = new_errors;
				others = new_others;

				if filtered_messages.is_empty()
				{
					break;
				}
			}
			Ok(Command::NoChange) =>
			{}
			Err(_) => break,
		}
	}

	child.wait()?;

	cleanup_tui(&mut terminal)?;

	Ok(())
}
