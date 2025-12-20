use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Wrap, Block, Borders, Paragraph};
use ratatui::text::{Line, Span};

use crossterm::cursor::{Hide, Show};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen, enable_raw_mode, disable_raw_mode};
use crossterm::event::{self, Event, KeyCode};
use crossterm::execute;

use std::io::Stdout;

use syntect::parsing::SyntaxSet;
use syntect::highlighting::ThemeSet;

use crate::highlighting::highlight_message;

pub enum Command
{
	SwitchError(usize),
	IgnoreWarnings,
	Rebuild,
	NoChange,
	Quit,
}

pub fn setup_tui() -> anyhow::Result<(Terminal<CrosstermBackend<Stdout>>, SyntaxSet, ThemeSet)>
{
	let mut stdout = Box::new(std::io::stdout());

	enable_raw_mode()?;

	execute!(stdout, EnterAlternateScreen, Hide)?;

	let backend = CrosstermBackend::new(*stdout);
	let mut terminal = Terminal::new(backend)?;

	terminal.clear()?;

	let syntax_set = SyntaxSet::load_defaults_newlines();
	let theme_set = ThemeSet::load_defaults();

	Ok((terminal, syntax_set, theme_set))
}

pub fn cleanup_tui(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> anyhow::Result<()>
{
	// terminal.clear()?;
	disable_raw_mode()?;

	execute!(terminal.backend_mut(), LeaveAlternateScreen, Show)?;

	terminal.show_cursor()?;
	// terminal.clear()?;
	terminal.flush()?;
	Ok(())
}

pub fn control_tui(index: usize, total: usize) -> anyhow::Result<Command>
{
	if let Event::Key(key) = event::read()?
	{
		Ok(match key.code
		{
			KeyCode::Char('i') => Command::IgnoreWarnings,
			KeyCode::Char('r') => Command::Rebuild,
			KeyCode::Char('q') => Command::Quit,
			KeyCode::Char('j') | KeyCode::Down | KeyCode::Char('l') | KeyCode::Right =>
			{
				if index + 1 < total
				{
					Command::SwitchError(index + 1)
				}
				else
				{
					Command::NoChange
				}
			}
			KeyCode::Char('k') | KeyCode::Up | KeyCode::Char('h') | KeyCode::Left =>
			{
				if index > 0
				{
					Command::SwitchError(index - 1)
				}
				else
				{
					Command::NoChange
				}
			}
			_ => Command::NoChange,
		})
	}
	else
	{
		Ok(Command::NoChange)
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
	warnings: usize,
	errors: usize,
	others: usize,
	ignore_warnings: bool,
) -> anyhow::Result<()>
{
	terminal.draw(|f| {
		let bottom_line: Line = Line::from(vec![
			Span::styled(
				format!("WARN {} ", warnings),
				Style::default().fg(Color::Yellow),
			),
			Span::styled(format!("ERR {} ", errors), Style::default().fg(Color::Red)),
			Span::styled(format!("OTH {} ", others), Style::default().fg(Color::Blue)),
			Span::raw(format!(
				" Ignore warnings: {} ",
				if ignore_warnings { "ON" } else { "OFF" }
			)),
			Span::raw("h/← l/→ : navigate | i : toggle warnings | r : rebuild | q : quit"),
		]);

		let area = f.area();
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
			.title_bottom(bottom_line)
			.borders(Borders::ALL)
			.border_style(Style::default().fg(border_color));

		let highlighted = match highlight_message(&text, &syntax_set, &theme_set, theme)
		{
			Ok(spans) => spans,
			Err(e) => vec![Line::from(vec![Span::raw(format!(
				"Error generating highlighted text:\n{}",
				e
			))])],
		};
		let paragraph = Paragraph::new(highlighted)
			.block(block)
			.wrap(Wrap { trim: true });

		f.render_widget(paragraph, area);
	})?;

	Ok(())
}
