use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct ContributionDay {
    pub _date: String,
    #[serde(rename = "contributionCount")]
    pub contribution_count: u32,
    pub color: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Week {
    #[serde(rename = "contributionDays")]
    pub contribution_days: Vec<ContributionDay>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ContributionCalendar {
    #[serde(rename = "totalContributions")]
    pub total_contributions: u32,
    pub weeks: Vec<Week>,
}

#[derive(Debug, Deserialize)]
pub struct ContributionsCollection {
    #[serde(rename = "contributionCalendar")]
    pub contribution_calendar: ContributionCalendar,
}

#[derive(Debug, Deserialize)]
pub struct User {
    #[serde(rename = "contributionsCollection")]
    pub contributions_collection: ContributionsCollection,
}

#[derive(Debug, Deserialize)]
pub struct Data {
    #[serde(rename = "user")]
    pub user: User,
}

#[derive(Debug, Deserialize)]
pub struct GraphQLResponse {
    pub data: Data,
}
