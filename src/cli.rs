use clap::{Parser, Subcommand};

/// The CLI arguments for the application.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    #[command(subcommand)]
    pub subcommand: Option<CliSubcommand>,
    #[command(flatten)]
    telegram_cli: TelegramCli,
}

#[derive(Subcommand, Debug)]
pub enum CliSubcommand {
    /// Remove config, data, and/or log directories (with confirmation unless --yes).
    Clear {
        #[arg(short, long)]
        config: bool,
        #[arg(short, long)]
        data: bool,
        #[arg(short, long)]
        logs: bool,
        #[arg(long)]
        all: bool,
        #[arg(short, long)]
        yes: bool,
    },
    /// Copy default config files to the user config directory (only missing files unless --force).
    InitConfig {
        #[arg(long)]
        force: bool,
    },
}

impl CliArgs {
    /// Get the Telegram CLI arguments.
    pub fn telegram_cli(&self) -> &TelegramCli {
        &self.telegram_cli
    }
}

#[derive(Parser, Debug)]
/// The Telegram commands.
pub struct TelegramCli {
    #[arg(
        short,
        long,
        visible_alias = "lo",
        help = "Logout from the tgt",
        default_value_t = false
    )]
    logout: bool,

    #[arg(
        short,
        long,
        visible_alias = "sm",
        number_of_values = 2,
        value_names = &["CHAT_NAME", "MESSAGE"],
        help = "Send a message to a chat"
    )]
    send_message: Option<Vec<String>>,
}

impl TelegramCli {
    /// Get the logout flag.
    pub fn logout(&self) -> bool {
        self.logout
    }
    /// Get the send message arguments.
    pub fn send_message(&self) -> Option<&Vec<String>> {
        self.send_message.as_ref()
    }
}

// #[derive(Parser, Debug)]
// // #[derive(Subcommand, Debug)]
// pub enum Telegram {
//     Test(TelegramStartSubcommand),
//     Add { name: Option<String> }
// }
