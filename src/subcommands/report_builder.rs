//! A bit dirty, but quick way to generate an HTML report. 
//! Feel free to make your own report from the json definition :).

use std::{fs, path::Path};

use clap::Parser;

use crate::{utils, Report, ReportDependencyEntry};

/// Builds a simple HTML report from the output file of the `unused-features analyze` subcommand.
#[derive(Parser, Debug, Clone, Default)]
#[clap(author, version, about)]
#[clap(setting = clap::AppSettings::DeriveDisplayOrder)]
pub struct ReportBuildingCommand {
    /// The absolute path to the json report generated by `cargo unused-features analyze`.
    #[clap(short = 'i', long = "input", value_parser)]
    pub input_json_path: String,
    /// The absolute report file path (including name and extension) to which the report will be written.
    /// Defaults to the parent directory of the json report provided with '-i' or '--input'.
    #[clap(short = 'o', long = "output", value_parser)]
    pub output_report_path: Option<String>,
    /// The log level (debug, info, warn, error, off). Defaults to info.
    #[clap(short = 'l', long = "l")]
    pub log_level: Option<String>,
}

impl ReportBuildingCommand {
    pub fn execute(self) -> anyhow::Result<()> {
        utils::initialize_logger(self.log_level);

        let report = Report::from(Path::new(&self.input_json_path))?;

        let mut total_features = 0;
        let mut total_removed_features = 0;
        let mut total_crates = 0;

        let mut body = String::new();

        log::info!("Start building HTML report.");

        for (workspace_crate_name, workspace_crate) in report.workspace_crates {
            total_crates += 1;

            let mut dependencies_html_rows = String::new();

            for (dependency_name, dependency) in workspace_crate.dependencies {
                total_features += dependency.original_features.len();
                total_removed_features += dependency.successfully_removed_features.len();

                dependencies_html_rows
                    .push_str(&dependency_html_table(dependency_name, dependency));
            }

            let html_table = dependencies_table(dependencies_html_rows);
            body.push_str(&collapsable_header(
                html_table,
                workspace_crate_name,
                workspace_crate.full_path,
            ));
        }

        let totals_overview_html =
            totals_overview_table(total_crates, total_features, total_removed_features);

        let html_report = html_report(totals_overview_html, body);

        log::info!("Finished building HTML report.");

        let report_path = self.output_report_path.unwrap_or_else(|| {
            Path::new(&self.input_json_path)
                .parent()
                .unwrap()
                .join("report.html")
                .display()
                .to_string()
        });

        fs::write(&report_path, html_report)?;

        log::info!("Written HTML report to {}", report_path);

        Ok(())
    }
}

fn totals_overview_table(
    total_features: usize,
    total_removed_features: usize,
    total_crates: usize,
) -> String {
    format!(
        "
    <table class=\"styled-table\">
       <tr>
           <th>Total Crates</th>
           <th>Total Features</th>
           <th>Total Potential Removable Features</th>
       </tr>
       <tr>
        <td>{}</td>
        <td>{}</td>
        <td>{}</td>
       </tr>
    </table>
       ",
        total_features, total_removed_features, total_crates
    )
}

fn collapsable_header(table: String, crate_name: String, full_path: String) -> String {
    format!(
        " 
        <button type='button' class='collapsible'><h3>{}</h3></button>
        <div class='content'>
            <i style='margin: 5px' class='styled-table'>{}</i>
            {}
        </div>",
        crate_name, full_path, table
    )
}

fn dependency_html_table(crate_name: String, dependency: ReportDependencyEntry) -> String {
    let original_features = dependency
        .original_features
        .into_iter()
        .collect::<Vec<String>>()
        .join(", ");
    let successfully_removed_features = dependency
        .successfully_removed_features
        .into_iter()
        .collect::<Vec<String>>()
        .join(", ");
    let unsuccessfully_removed_features = dependency
        .unsuccessfully_removed_features
        .into_iter()
        .collect::<Vec<String>>()
        .join(", ");

    let dependency_html = format!(
        "
        <tr>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
        </tr>",
        crate_name,
        original_features,
        successfully_removed_features,
        unsuccessfully_removed_features
    );

    dependency_html
}

fn dependencies_table(dependency_rows: String) -> String {
    format!(
        "
        <table class=\"styled-table\">
        <tr>
            <th>Dependency</th>
            <th>Original</th>
            <th>Potential Removable</th>
            <th>Unremovable</th>
        </tr>
        {}
        </table>       
       ",
        dependency_rows
    )
}

fn html_report(totals_overview_html: String, html_body: String) -> String {
    format!(
        "
    <html>
        <head>
            <title>Unused Feature Flag Finder Report</title>
        
            <style>
                .styled-table {{
                    border-collapse: collapse;
                    margin: 25px 0;
                    font-size: 0.9em;
                    font-family: sans-serif;
                    min-width: 400px;
                    box-shadow: 0 0 20px rgba(0, 0, 0, 0.15);
                }}
                .styled-table thead tr {{
                    background-color: #009879;
                    color: #ffffff;
                    text-align: left;
                }}
                .styled-table th,
                .styled-table td {{
                    text-align: left;
                    padding: 12px 15px;
                }}
                .styled-table tbody tr {{
                    border-bottom: 1px solid #dddddd;
                }}                
                .styled-table tbody tr:nth-of-type(even) {{
                    background-color: #f3f3f3;
                }}                
                .styled-table tbody tr:last-of-type {{
                    border-bottom: 2px solid #009879;
                }}
                .collapsible {{
                    background-color: #eee;
                    color: #444;
                    cursor: pointer;
                    padding: 10px;
                    width: 100%;
                    border: none;
                    text-align: left;
                    outline: none;
                    font-size: 15px;
                    font-family: sans-serif;
                }}
                .active,
                .collapsible:hover {{
                    background-color: #ccc;
                }}
                .content {{
                    padding: 10px 18px;
                    display: none;
                    overflow: hidden;
                    background-color: #f1f1f1;
                }}
            </style>
        </head>
        
        <body>
            {}
            {}
        </body>
        
        </html>
        
        <script>
            var coll = document.getElementsByClassName(\"collapsible\");
            var i;
        
            for (i = 0; i < coll.length; i++) {{
                coll[i].addEventListener(\"click\", function () {{
                    this.classList.toggle(\"active\");
                    var content = this.nextElementSibling;
                    if (content.style.display === \"block\") {{
                        content.style.display = \"none\";
                    }} else {{
                        content.style.display = \"block\";
                    }}
                }});
            }}
        </script>
    </html>",
        totals_overview_html, html_body
    )
}
