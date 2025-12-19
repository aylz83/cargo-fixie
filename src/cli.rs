use clap::Parser;

#[derive(Parser)]
#[command(name = "cargo")]
#[command(bin_name = "cargo")]
#[command(styles = CLAP_STYLING)]
pub enum Cli
{
	Fixie(FixieArgs),
}

pub const CLAP_STYLING: clap::builder::styling::Styles = clap::builder::styling::Styles::styled()
	.header(clap_cargo::style::HEADER)
	.usage(clap_cargo::style::USAGE)
	.literal(clap_cargo::style::LITERAL)
	.placeholder(clap_cargo::style::PLACEHOLDER)
	.error(clap_cargo::style::ERROR)
	.valid(clap_cargo::style::VALID)
	.invalid(clap_cargo::style::INVALID);

#[derive(clap::Args)]
#[command(version, about, long_about = None)]
pub struct FixieArgs
{
	/// Disable warnings being included in the TUI output
	#[arg(short, long)]
	pub ignore_warnings: bool,

	/// Set the syntax highlighting theme
	#[arg(short, long, default_value = "base16-ocean.dark")]
	pub theme: String,
}
