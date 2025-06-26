use crate::structs::{Config, ContributionCalendar, ContributionDay, GraphQLResponse, Week};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue, USER_AGENT};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

mod structs;

const CONFIG_DIR_NAME: &str = "contrib";
const CONFIG_FILE_NAME: &str = "config.json";

#[tokio::main]
async fn main() {
    let config_path = get_config_path();
    let config_file_exists = check_config_file_exists(&config_path);
    if !config_file_exists {
        println!(
            "No config file found. Please enter your GitHub username and personal access token."
        );
    }

    let (gh_username, personal_access_token) = if config_file_exists {
        read_config_file(&config_path)
    } else {
        let (gh_username, personal_access_token) = prompt_user_for_config();
        create_config_file(&config_path, &gh_username, &personal_access_token);
        println!(
            "Config file created successfully at: {}",
            config_path.display()
        );
        (gh_username, personal_access_token)
    };

    let contribution_calendar = get_contributions(&gh_username, &personal_access_token).await;
    print_week_squares(&contribution_calendar.weeks);
    print_contributions_and_streak(&contribution_calendar);
}

fn get_config_path() -> PathBuf {
    let config_dir = dirs::config_dir()
        .expect("Could not determine config directory")
        .join(CONFIG_DIR_NAME);

    // Create the config directory if it doesn't exist
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).expect("Failed to create config directory");
    }

    config_dir.join(CONFIG_FILE_NAME)
}

fn check_config_file_exists(config_path: &Path) -> bool {
    config_path.exists()
}

fn read_config_file(config_path: &Path) -> (String, String) {
    let config_file_contents = fs::read_to_string(config_path).expect("Failed to read config file");
    let config: Config =
        serde_json::from_str(&config_file_contents).expect("Failed to parse config file");
    (config.gh_username, config.personal_access_token)
}

fn prompt_user_for_config() -> (String, String) {
    println!("Enter your GitHub username: ");
    let mut gh_username = String::new();
    io::stdin()
        .read_line(&mut gh_username)
        .expect("Failed to read line");
    gh_username = gh_username.trim().to_string();

    println!("Enter your personal access token: ");
    let mut personal_access_token = String::new();
    io::stdin()
        .read_line(&mut personal_access_token)
        .expect("Failed to read line");
    personal_access_token = personal_access_token.trim().to_string();

    (gh_username, personal_access_token)
}

fn create_config_file(config_path: &Path, gh_username: &str, personal_access_token: &str) {
    let config = Config {
        gh_username: gh_username.to_string(),
        personal_access_token: personal_access_token.to_string(),
    };
    fs::write(
        config_path,
        serde_json::to_string(&config).expect("Failed to serialize config"),
    )
    .expect("Failed to write config file");
}

async fn get_contributions(gh_username: &str, personal_access_token: &str) -> ContributionCalendar {
    let query = format!(
        r#"
        {{
          user(login: "{}") {{
            contributionsCollection {{
              contributionCalendar {{
                totalContributions
                weeks {{
                  contributionDays {{
                    date
                    contributionCount
                    color
                  }}
                }}
              }}
            }}
          }}
        }}
        "#,
        gh_username
    );

    let json = serde_json::json!({"query": query});

    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("bearer {}", personal_access_token))
            .expect("Failed to create authorization header"),
    );
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(USER_AGENT, HeaderValue::from_static("contrib/1.0"));
    let client = reqwest::Client::new();
    let res = client
        .post("https://api.github.com/graphql")
        .headers(headers)
        .json(&json)
        .send()
        .await
        .expect("Failed to send request");

    let body: GraphQLResponse = res.json().await.expect("Failed to parse response");
    let contribution_calendar = &body
        .data
        .user
        .contributions_collection
        .contribution_calendar;

    contribution_calendar.clone()
}
fn github_color_to_ansi(hex: &str) -> u8 {
    match hex {
        "#ebedf0" => 255, // white
        "#9be9a8" => 120, // light green
        "#40c463" => 34,  // green
        "#30a14e" => 28,  // darker green
        "#216e39" => 22,  // darkest green
        _ => 255,         // default to white
    }
}

fn print_square(square_color: u8) {
    print!("\x1b[48;5;{}m  \x1b[0m", square_color);
}

fn get_contributions_for_today(weeks: &[Week]) -> u32 {
    weeks
        .last()
        .unwrap()
        .contribution_days
        .last()
        .unwrap()
        .contribution_count
}

fn get_streak(weeks: &[Week]) -> u32 {
    let mut streak = 0;

    let all_days: Vec<&ContributionDay> = weeks
        .iter()
        .flat_map(|week| week.contribution_days.iter())
        .collect();

    for day in all_days.iter().rev() {
        if day.contribution_count > 0 {
            streak += 1;
        } else {
            break;
        }
    }
    streak
}

fn print_week_squares(weeks: &[Week]) {
    println!();
    for day in 0..7 {
        for week in weeks.iter() {
            if let Some(square) = week.contribution_days.get(day) {
                let ansi_color = github_color_to_ansi(&square.color);
                print_square(ansi_color);
            } else {
                print!("  ");
            }
        }
        println!();
    }
}

fn print_contributions_and_streak(contribution_calendar: &ContributionCalendar) {
    println!();
    println!(
        "Contributions for today: {}",
        get_contributions_for_today(&contribution_calendar.weeks)
    );
    println!();
    println!(
        "Total contributions in the last year: {}",
        &contribution_calendar.total_contributions
    );
    println!();
    println!(
        "Current streak: {} days",
        get_streak(&contribution_calendar.weeks)
    );
}
