use crate::structs::{ContributionCalendar, GraphQLResponse, Week};
use dotenv::dotenv;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue, USER_AGENT};
use std::env;

mod structs;

#[tokio::main]
async fn main() {
    dotenv().ok();

    if env::var("GITHUB_PERSONAL_ACCESS_TOKEN").is_err() {
        panic!(
            "GITHUB_PERSONAL_ACCESS_TOKEN is not set. Please set it in the .env file for this application to work."
        );
    }

    if env::var("GITHUB_USERNAME").is_err() {
        panic!(
            "GITHUB_USERNAME is not set. Please set it in the .env file for this application to work."
        );
    }
    let gh_username = env::var("GITHUB_USERNAME").unwrap();
    let personal_access_token = env::var("GITHUB_PERSONAL_ACCESS_TOKEN").unwrap();

    let total_contributions = get_contributions(&gh_username, &personal_access_token).await;
    print_week_squares(&total_contributions.weeks);
    println!(
        "Contributions for today: {}",
        get_contributions_for_today(&total_contributions.weeks)
    );
    println!(
        "Total contributions in the last year: {}",
        &total_contributions.total_contributions
    );
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

fn print_week_squares(weeks: &Vec<Week>) {
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

fn get_contributions_for_today(weeks: &Vec<Week>) -> u32 {
    weeks
        .last()
        .unwrap()
        .contribution_days
        .last()
        .unwrap()
        .contribution_count
}
