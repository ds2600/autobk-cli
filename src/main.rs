use clap::{Parser, Subcommand};
use mysql::*;
use mysql::prelude::*;
use config::{Config, File, FileFormat};

/// AutoBk-CLI: Interact with AutoBk database from the command line.
/// 
/// Examples:
///     autobk add --name "DCM-1" --device_type "DCM" --ipv4 "192.168.1.10" --day 3 --hour 12 --weeks 2
///     autobk modify -n "APEX-100" -t "APEX" -r "192.168.1.11" -d 5 -h 14 -w 0

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(after_help = "For more information, visit https://github.com/ds2600/autobk-cli")]
struct Opts {
    #[clap(subcommand)]
    action: Action,
}

#[derive(Subcommand)]
enum Action {
    Add {
        #[clap(flatten)]
        data: AddData,
    },
    Modify {
        #[clap(flatten)]
        data: CommonData,
    },
    Delete {
        #[clap(flatten)]
        data: CommonData,
    },
    Backup {
        #[clap(flatten)]
        data: BackupData,
    },
    Get {
        #[clap(flatten)]
        data: QueryData,
    },
}


#[derive(Parser)]
#[derive(Debug)]
struct QueryData {
    #[clap(short = 'n', long, value_name = "NAME")]
    name: String,
}


#[derive(Parser)]
#[derive(Debug)]
struct AddData {
    #[clap(short = 'n', long, value_name = "NAME")]
    name: String,
    #[clap(short = 't', long, value_name = "DEVICE_TYPE")]
    device_type: String,
    #[clap(short = 'i', long, value_name = "IPv4_ADDRESS")]
    ipv4: String,
    #[clap(short = 'd', long, value_name = "DOW_INTEGER")]
    day: u8,
    #[clap(short = 'r', long, value_name = "HOUR")]
    hour: u8,
    #[clap(short = 'w', long, value_name = "WEEKS")]
    weeks: u8,
}

#[derive(Parser)]
#[derive(Debug)]
struct CommonData {
    #[clap(short = 'n', long, value_name = "NAME")]
    name: String,
    #[clap(short = 't', long, value_name = "DEVICE_TYPE")]
    device_type: String,
    #[clap(short = 'i', long, value_name = "IPv4_ADDRESS")]
    ipv4: String,
    #[clap(short = 'd', long, value_name = "DOW_INTEGER")]
    day: u8,
    #[clap(short = 'r', long, value_name = "HOUR")]
    hour: u8,
    #[clap(short = 'w', long, value_name = "WEEKS")]
    weeks: u8,

}

#[derive(Parser)]
#[derive(Debug)]
struct NameData {
    #[clap(short = 'n', long, value_name = "NAME")]
    name: String,
}

#[derive(Parser)]
#[derive(Debug)]
struct IdData {
    #[clap(short = 'd', long, value_name = "DEVICE_ID")]
    device_id: String,
}

#[derive(Parser)]
#[derive(Debug)]
enum BackupData {
    NameData(NameData),
    IdData(IdData),
}

fn main() {
    let opts: Opts = Opts::parse();

    match opts.action {
        Action::Add { data } => {
            if data.name.is_empty() || data.device_type.is_empty() || data.ipv4.is_empty() {
                println!("Invalid data");
                return;
            }

            let mut settings = Config::default();
            settings
                .merge(File::from_str(include_str!("Settings.toml"), FileFormat::Toml))
                .unwrap();

            let db = OptsBuilder::new()
                .ip_or_hostname(Some(settings.get::<String>("db_host").unwrap()))
                .db_name(Some(settings.get::<String>("db_name").unwrap()))
                .user(Some(settings.get::<String>("db_user").unwrap()))
                .pass(Some(settings.get::<String>("db_pass").unwrap()));

            let pool = match Pool::new(db) {
                Ok(pool) => pool,
                Err(err) => {
                    println!("Error: {}", err);
                    return;
                }
            };

            let mut conn = match pool.get_conn() {
                Ok(conn) => conn,
                Err(err) => {
                    println!("Error: {}", err);
                    return;
                }
            };

            let sql = "INSERT INTO Device (sName, sType, sIP, iAutoDay, iAutoHour, iAutoWeeks) VALUES (:name, :device_type, :ipv4, :day, :hour, :weeks)";

            let params = params! {
                "name" => &data.name,
                "device_type" => data.device_type,
                "ipv4" => data.ipv4,
                "day" => data.day,
                "hour" => data.hour,
                "weeks" => data.weeks,
            };

            match conn.exec_drop(sql, params) {
                Ok(_) => println!("{} added successfully", data.name),
                Err(err) => println!("Error: {}", err),
            }
        }
        Action::Modify { data } => {
            println!("Modifying data.");
        }
        Action::Delete { data } => {
            println!("Deleting data");
        }
        Action::Backup { data } => {
            if data.name.is_empty() {
                println!("Invalid name");
                return;
            }
        }
        Action::Get { data } => {
            if data.name.is_empty() {
                println!("Invalid name");
                return;
            }

            let mut settings = Config::default();
            settings
                .merge(File::from_str(include_str!("Settings.toml"), FileFormat::Toml))
                .unwrap();

            let db = OptsBuilder::new()
                .ip_or_hostname(Some(settings.get::<String>("db_host").unwrap()))
                .db_name(Some(settings.get::<String>("db_name").unwrap()))
                .user(Some(settings.get::<String>("db_user").unwrap()))
                .pass(Some(settings.get::<String>("db_pass").unwrap()));

            let pool = match Pool::new(db) {
                Ok(pool) => pool,
                Err(err) => {
                    println!("Error: {}", err);
                    return;
                }
            };

            let mut conn = match pool.get_conn() {
                Ok(conn) => conn,
                Err(err) => {
                    println!("Error: {}", err);
                    return;
                }
            };

            let sql = "SELECT kSelf, sName FROM Device WHERE sName = :name";

            let params = params! {
                "name" => &data.name,
            };

            let result: mysql::QueryResult<mysql::Binary> = conn.exec_iter(sql, params).unwrap();

            for row_result in result {
                let row = row_result.unwrap();
                let k_self: i32 = row.get("kSelf").unwrap();
                let s_name: String = row.get("sName").unwrap();

                println!("Name: {}, ID: {}", s_name, k_self);
            }
        }
    }
}