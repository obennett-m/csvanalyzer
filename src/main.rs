use clap::Parser;
use csvanalyzertool::{Config, CsvAnalyzer, DbConfig};
use std::path::Path;

/// CSV Analyzer - Analyze CSV files for contact import
#[derive(Parser, Debug)]
#[command(name = "csvanalyzertool")]
#[command(about = "Analyze CSV files for contact import, detecting format and validating data")]
#[command(
    version,
    after_help = "Tool will print 10 records of CSV file in JSON format or empty string if invalid CSV file.\nIn case of unexpected exception, tool will print message started with: \"Error: <message>\""
)]
struct Args {
    /// Account ID (required for database queries)
    #[arg(short = 'a', long = "akid", required = true)]
    akid: i64,

    /// User locale (e.g., "en_US")
    #[arg(short = 'l', long = "locale", required = true)]
    locale: String,

    /// Path to CSV file
    #[arg(short = 'f', long = "filename", required = true)]
    filename: String,

    /// Path to config file (default: /etc/mailjet.conf)
    #[arg(short = 'c', long = "config")]
    config_file: Option<String>,

    /// PostgreSQL host (overrides config file and env vars)
    #[arg(long = "db-host")]
    db_host: Option<String>,

    /// PostgreSQL port (default: 5432)
    #[arg(long = "db-port")]
    db_port: Option<u16>,

    /// PostgreSQL database name
    #[arg(long = "db-name")]
    db_name: Option<String>,

    /// PostgreSQL user
    #[arg(long = "db-user")]
    db_user: Option<String>,

    /// PostgreSQL password (can also use PGPASSWORD env var)
    #[arg(long = "db-password")]
    db_password: Option<String>,

    /// Number of lines to scan (default: 1000)
    #[arg(long = "scan-lines")]
    scan_lines: Option<usize>,

    /// Number of data rows to return (default: 10)
    #[arg(long = "return-lines")]
    return_lines: Option<usize>,
}

fn main() {
    let args = Args::parse();

    // Validate file exists
    if !Path::new(&args.filename).exists() {
        eprintln!(
            "{{\"Error\":2,\"ErrorMsgUser\":\"Could not get a sample for analyze. Is file empty?\",\"ErrorMsgInternal\":\"File not found: {}\"}}",
            args.filename
        );
        std::process::exit(1);
    }

    // Build database config
    let db_config = match build_db_config(&args) {
        Ok(config) => config,
        Err(e) => {
            eprintln!(
                "{{\"Error\":3,\"ErrorMsgUser\":\"Database configuration error\",\"ErrorMsgInternal\":\"{}\"}}",
                e
            );
            std::process::exit(1);
        }
    };

    // Build main config
    let mut config = Config::new_with_db(args.akid, args.locale, args.filename, db_config);

    if let Some(scan_lines) = args.scan_lines {
        config.scan_lines = scan_lines;
    }
    if let Some(return_lines) = args.return_lines {
        config.return_lines = return_lines;
    }

    // Run analyzer
    let mut analyzer = CsvAnalyzer::new(config);
    let result = analyzer.analyze();

    // Output JSON to stdout
    println!("{}", result);
}

fn build_db_config(args: &Args) -> Result<DbConfig, String> {
    // Priority: CLI args > config file > environment variables
    let mut db_config: Option<DbConfig> = None;

    // Try loading from config file first
    if let Some(ref config_path) = args.config_file {
        if Path::new(config_path).exists() {
            db_config = match DbConfig::from_file(config_path) {
                Ok(config) => Some(config),
                Err(e) => return Err(format!("Failed to load config file {}: {}", config_path, e)),
            };
        } else {
            return Err(format!("Config file not found: {}", config_path));
        }
    } else {
        // Try default config file
        let default_config = "/etc/mailjet.conf";
        if Path::new(default_config).exists() {
            db_config = DbConfig::from_file(default_config).ok();
        }
    }

    // Try environment variables if no config file loaded
    if db_config.is_none() {
        db_config = DbConfig::from_env().ok();
    }

    // If still no config, check if we have all required CLI args to build one
    if db_config.is_none() {
        let has_all_cli_args = args.db_host.is_some()
            && args.db_name.is_some()
            && args.db_user.is_some()
            && args.db_password.is_some();

        if !has_all_cli_args {
            return Err(
                "Database configuration not found. Please provide configuration via:\n\
                 - Config file (--config or /etc/mailjet.conf)\n\
                 - Environment variables (PGHOST, PGPORT, PGDATABASE, PGUSER, PGPASSWORD)\n\
                 - CLI arguments (--db-host, --db-name, --db-user, --db-password)"
                    .to_string(),
            );
        }

        // Build config from CLI arguments
        db_config = Some(DbConfig::new(
            args.db_host.clone().unwrap(),
            args.db_port.unwrap_or(5432),
            args.db_name.clone().unwrap(),
            args.db_user.clone().unwrap(),
            args.db_password.clone().unwrap(),
        ));
    }

    // Get the config (we know it exists at this point)
    let mut config = db_config.unwrap();

    // CLI arguments override everything
    if let Some(ref host) = args.db_host {
        config.host = host.clone();
    }
    if let Some(port) = args.db_port {
        config.port = port;
    }
    if let Some(ref name) = args.db_name {
        config.database = name.clone();
    }
    if let Some(ref user) = args.db_user {
        config.user = user.clone();
    }
    if let Some(ref password) = args.db_password {
        config.password = password.clone();
    }

    Ok(config)
}
