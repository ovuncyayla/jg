use clap::Parser;
use std::path::PathBuf;
use jg::{generate_json, Config};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct JGArgs {
    /// Sets a custom config file
    #[arg(short, long, value_name = "Config File", value_parser = clap::value_parser!(PathBuf),  default_value = "config.yaml")]
    config: Option<PathBuf>,
}

fn main() {
    let env = env_logger::Env::default()
        .filter_or("JG_LOG", "info")
        .write_style_or("JG_LOG_STYLE", "always");

    env_logger::init_from_env(env);

    let args = JGArgs::parse();
    let mut config: Config = Config::from(args.config.unwrap());

    let output = generate_json(&mut config);

    println!(
        "{}",
        serde_json::to_string_pretty(&output.unwrap())
            .unwrap()
            .to_string()
    );
}
