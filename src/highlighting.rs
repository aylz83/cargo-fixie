use ratatui::text::{Line, Span};
use ratatui::style::{Style, Color};

use syntect::parsing::SyntaxSet;
use syntect::highlighting::ThemeSet;

pub fn highlight_message(
	code: &str,
	syntax_set: &SyntaxSet,
	theme_set: &ThemeSet,
	theme: &String,
) -> anyhow::Result<Vec<Line<'static>>>
{
	let syntax = syntax_set
		.find_syntax_by_extension("rs")
		.ok_or(anyhow::anyhow!("Unable to find Rust themes"))?;

	// Default to base16-ocean.dark when an unknown theme is attempted to be used.
	let mut highlighter = syntect::easy::HighlightLines::new(
		syntax,
		&theme_set
			.themes
			.get(theme)
			.unwrap_or(&theme_set.themes["base16-ocean.dark"]),
	);

	let mut result = Vec::new();

	for line in code.lines()
	{
		let ranges = highlighter
			.highlight_line(line, syntax_set)
			.map_err(|e| anyhow::anyhow!("Highlighting failed: {}", e))?;

		let spans: Vec<Span<'static>> = ranges
			.iter()
			.map(|(style, text)| {
				let fg = Color::Rgb(style.foreground.r, style.foreground.g, style.foreground.b);
				Span::styled(text.to_string(), Style::default().fg(fg))
			})
			.collect();

		result.push(Line::from(spans));
	}

	Ok(result)
}
