use std::fmt::Write;

use humansize::{format_size_i, BINARY};
use tabled::{locator::ByColumnName, Alignment, Modify, Style, Table, Tabled};

use crate::defrag::Browser;

#[derive(Debug, Tabled)]
struct DatabaseReport {
    #[tabled(rename = "Database")]
    path: String,

    #[tabled(rename = "Defrag")]
    defrag: String,

    #[tabled(rename = "Before")]
    size_before: String,

    #[tabled(rename = "After")]
    size_after: String,

    #[tabled(rename = "Changed")]
    changed: String,

    #[tabled(rename = "Changed %")]
    percent: String,
}

impl std::fmt::Display for Browser {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.database_lists.is_none() {
            return write!(f, "{}: NO PROFILE FOUND", self.name);
        }

        let mut output = String::new();
        for database_list in self.database_lists.as_ref().unwrap() {
            if database_list.databases.is_empty() {
                write!(
                    &mut output,
                    "{browser_name}: {profile_path}/\nNO DATABASE FOUND",
                    browser_name = self.name,
                    profile_path = database_list.profile_path.display()
                )?;
                continue;
            }

            writeln!(
                &mut output,
                "\n{browser_name}: {profile_path}/",
                browser_name = self.name,
                profile_path = database_list.profile_path.display()
            )?;

            // Create table of database files
            let mut db_table: Vec<DatabaseReport> = Vec::new();
            let mut total_before: f64 = 0.0;
            let mut total_after: f64 = 0.0;
            let mut total_changed: f64 = 0.0;
            for db in &database_list.databases {
                let path: String = db
                    .path
                    .strip_prefix(&database_list.profile_path)
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string();

                let defrag: String = match db.defrag {
                    true => "Yes".to_string(),
                    false => "No".to_string(),
                };

                let size_before: String = db.size_before.map_or("N/A".to_string(), |s| {
                    total_before += s as f64;
                    format_size_i(s, BINARY)
                });

                let size_after: String = db.size_after.map_or("N/A".to_string(), |s| {
                    total_after += s as f64;
                    format_size_i(s, BINARY)
                });

                let changed: String = {
                    if db.size_before.is_none() || db.size_after.is_none() {
                        "N/A".to_string()
                    } else {
                        let diff: f64 =
                            db.size_after.unwrap() as f64 - db.size_before.unwrap() as f64;
                        total_changed += diff;
                        format_size_i(diff, BINARY)
                    }
                };

                let percent: String = {
                    if db.size_before.is_none() || db.size_after.is_none() {
                        "N/A".to_string()
                    } else {
                        let diff: f64 =
                            db.size_after.unwrap() as f64 - db.size_before.unwrap() as f64;
                        let percent: f64 = (diff * 100.0_f64) / (db.size_before.unwrap() as f64);
                        format!("{percent:.2} %")
                    }
                };

                db_table.push(DatabaseReport {
                    path,
                    defrag,
                    size_before,
                    size_after,
                    changed,
                    percent,
                });
            }

            let total_percent: String = {
                let percent: f64 = (total_after - total_before) * 100.0_f64 / total_before;
                format!("{percent:.2} %")
            };

            db_table.push(DatabaseReport {
                path: "".to_string(),
                defrag: "".to_string(),
                size_before: format_size_i(total_before, BINARY),
                size_after: format_size_i(total_after, BINARY),
                changed: format_size_i(total_changed, BINARY),
                percent: total_percent,
            });

            let mut table = Table::new(db_table);
            table
                .with(Style::markdown())
                .with(Modify::new(ByColumnName::new("Database")).with(Alignment::left()))
                .with(Modify::new(ByColumnName::new("Defrag")).with(Alignment::center()))
                .with(Modify::new(ByColumnName::new("Before")).with(Alignment::right()))
                .with(Modify::new(ByColumnName::new("After")).with(Alignment::right()))
                .with(Modify::new(ByColumnName::new("Changed")).with(Alignment::right()))
                .with(Modify::new(ByColumnName::new("Changed %")).with(Alignment::right()));
            writeln!(&mut output, "{table}")?;
        }

        write!(f, "{output}")
    }
}
