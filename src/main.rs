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

	let messages = parse_build_output(reader, cli.ignore_warnings)?;

	child.wait()?;

	if messages.is_empty()
	{
		return Ok(());
	}

	let mut index = 0;

	let (mut terminal, syntax_set, theme_set) = setup_tui()?;

	loop
	{
		let message = &messages[index];
		render_message(
			&mut terminal,
			&syntax_set,
			&theme_set,
			&cli.theme,
			message,
			index,
			messages.len(),
		)?;

		match control_tui(index, messages.len())
		{
			Ok(Some(new_index)) => index = new_index,
			Ok(None) => break,
			Err(_) => break,
		}
	}

	cleanup_tui(&mut terminal)?;

	Ok(())
}
