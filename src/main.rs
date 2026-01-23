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
    let db_config = build_db_config(&args);

    // Build main config
    let mut config = Config::new(args.akid, args.locale, args.filename).with_db_config(db_config);

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

fn build_db_config(args: &Args) -> DbConfig {
    // Start with environment variables
    let mut db_config = DbConfig::from_env();

    // Try loading from config file if specified
    if let Some(ref config_path) = args.config_file {
        if Path::new(config_path).exists() {
            if let Ok(file_config) = DbConfig::from_file(config_path) {
                db_config = file_config;
            } else {
                eprintln!("Warning: Could not parse config file: {}", config_path);
            }
        }
    } else {
        // Try default config file
        let default_config = "/etc/mailjet.conf";
        if Path::new(default_config).exists() {
            if let Ok(file_config) = DbConfig::from_file(default_config) {
                db_config = file_config;
            }
        }
    }

    // CLI arguments override everything
    if let Some(ref host) = args.db_host {
        db_config.host = host.clone();
    }
    if let Some(port) = args.db_port {
        db_config.port = port;
    }
    if let Some(ref name) = args.db_name {
        db_config.database = name.clone();
    }
    if let Some(ref user) = args.db_user {
        db_config.user = user.clone();
    }
    if let Some(ref password) = args.db_password {
        db_config.password = password.clone();
    }

    db_config
}
