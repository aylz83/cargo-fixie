use tui::Terminal;
use tui::backend::CrosstermBackend;
use tui::style::{Color, Style};
use tui::widgets::{Wrap, Block, Borders, Paragraph};
use tui::text::{Spans, Span};

use crossterm::terminal::{enable_raw_mode, disable_raw_mode};
use crossterm::event::{self, Event, KeyCode};

use std::io::Stdout;

use syntect::parsing::SyntaxSet;
use syntect::highlighting::ThemeSet;

use crate::highlighting::highlight_message;

pub fn setup_tui() -> anyhow::Result<(Terminal<CrosstermBackend<Stdout>>, SyntaxSet, ThemeSet)>
{
	let stdout = Box::new(std::io::stdout());

	enable_raw_mode()?;

	let backend = CrosstermBackend::new(*stdout);
	let mut terminal = Terminal::new(backend)?;

	terminal.clear()?;

	let syntax_set = SyntaxSet::load_defaults_newlines();
	let theme_set = ThemeSet::load_defaults();

	Ok((terminal, syntax_set, theme_set))
}

pub fn cleanup_tui(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> anyhow::Result<()>
{
	terminal.clear()?;
	disable_raw_mode()?;
	Ok(())
}

pub fn control_tui(index: usize, total: usize) -> anyhow::Result<Option<usize>>
{
	if let Event::Key(key) = event::read()?
	{
		Ok(match key.code
		{
			KeyCode::Char('q') => return Ok(None),
			KeyCode::Char('j') | KeyCode::Down | KeyCode::Char('l') | KeyCode::Right =>
			{
				if index + 1 < total
				{
					Some(index + 1)
				}
				else
				{
					Some(index)
				}
			}
			KeyCode::Char('k') | KeyCode::Up | KeyCode::Char('h') | KeyCode::Left =>
			{
				if index > 0
				{
					Some(index - 1)
				}
				else
				{
					Some(index)
				}
			}
			_ => Some(index),
		})
	}
	else
	{
		Ok(Some(index))
	}
}

pub fn render_message(
	terminal: &mut Terminal<CrosstermBackend<Stdout>>,
	syntax_set: &SyntaxSet,
	theme_set: &ThemeSet,
	theme: &String,
	message: &(String, String),
	index: usize,
	total: usize,
) -> anyhow::Result<()>
{
	terminal.draw(|f| {
		let size = f.size();
		let (level, text) = message;

		// Determine styles
		let (border_color, title) = match level.as_str()
		{
			"warning" => (Color::Yellow, "WARNING"),
			"error" => (Color::Red, "  ERROR"),
			_ => (Color::Blue, "  OTHER"),
		};

		let block = Block::default()
			.title(format!("{} [{}/{}]", title, index + 1, total))
			.borders(Borders::ALL)
			.border_style(Style::default().fg(border_color));

		let highlighted = match highlight_message(&text, &syntax_set, &theme_set, theme)
		{
			Ok(spans) => spans,
			Err(e) => vec![Spans::from(Span::raw(format!(
				"Error generating highlighted text:\n{}",
				e
			)))],
		};
		let paragraph = Paragraph::new(highlighted)
			.block(block)
			.wrap(Wrap { trim: true });

		f.render_widget(paragraph, size);
	})?;

	Ok(())
}
