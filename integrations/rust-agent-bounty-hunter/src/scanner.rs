//! Bounty Scanner - Fetches and ranks open bounty leads from GitHub
//!
//! This module handles:
//! - Fetching issues with bounty-related labels
//! - Ranking by difficulty and reward potential
//! - Filtering by repository and status

use anyhow::{Result, Context};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct BountyLead {
    pub number: u64,
    pub title: String,
    pub body: String,
    pub labels: Vec<String>,
    pub reward_estimate: String,
    pub difficulty: String,
    pub url: String,
    pub repository: String,
}

impl BountyLead {
    pub fn score(&self) -> u64 {
        // Simple scoring: higher reward = higher score
        let reward_score = match self.reward_estimate.contains("100") {
            true => 100,
            true => 80,
            true => 50,
            _ => 20,
        };
        reward_score
    }
}

pub async fn scan_bounties(
    owner: &str,
    repo: &str,
    github_token: &str,
) -> Result<Vec<BountyLead>> {
    let client = reqwest::Client::new();
    
    // Fetch open issues with bounty labels
    let url = format!(
        "https://api.github.com/repos/{}/{}/issues?state=open&per_page=100",
        owner, repo
    );
    
    let response = client
        .get(&url)
        .header("Authorization", format!("token {}", github_token))
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
        .context("Failed to fetch issues")?
        .json::<serde_json::Value>()
        .await
        .context("Failed to parse issues")?;

    let issues = response.as_array()
        .context("Issues should be an array")?;

    let mut bounties: Vec<BountyLead> = Vec::new();

    for issue in issues {
        let labels = issue["labels"]
            .as_array()
            .map(|arr| arr.iter().filter_map(|l| l["name"].as_str().map(String::from)).collect())
            .unwrap_or_default();

        // Skip PRs
        if issue.get("pull_request").is_some() {
            continue;
        }

        // Check for bounty-related labels
        let has_bounty_label = labels.iter().any(|l| 
            l.to_lowercase().contains("bounty") || 
            l.to_lowercase().contains("reward") ||
            l.to_lowercase().contains("paid")
        );

        if has_bounty_label || is_reward_issue(&issue["body"].as_str().unwrap_or("")) {
            let (reward, difficulty) = parse_reward_info(&issue["body"].as_str().unwrap_or(""));
            
            bounties.push(BountyLead {
                number: issue["number"].as_u64().unwrap_or(0),
                title: issue["title"].as_str().unwrap_or("").to_string(),
                body: issue["body"].as_str().unwrap_or("").to_string(),
                labels,
                reward_estimate: reward,
                difficulty,
                url: issue["html_url"].as_str().unwrap_or("").to_string(),
                repository: format!("{}/{}", owner, repo),
            });
        }
    }

    // Sort by score (descending)
    bounties.sort_by(|a, b| b.score().cmp(&a.score()));

    Ok(bounties)
}

fn is_reward_issue(body: &str) -> bool {
    body.to_lowercase().contains("rtc") ||
    body.to_lowercase().contains("reward") ||
    body.to_lowercase().contains("bounty") ||
    body.to_lowercase().contains("payment")
}

fn parse_reward_info(body: &str) -> (String, String) {
    let body_lower = body.to_lowercase();
    
    let reward = if body_lower.contains("100") {
        "100+ RTC".to_string()
    } else if body_lower.contains("50") {
        "50 RTC".to_string()
    } else if body_lower.contains("25") {
        "25 RTC".to_string()
    } else if body_lower.contains("10") {
        "10 RTC".to_string()
    } else {
        "Unspecified".to_string()
    };

    let difficulty = if body_lower.contains("critical") || body_lower.contains("security") {
        "Critical".to_string()
    } else if body_lower.contains("high") {
        "High".to_string()
    } else if body_lower.contains("medium") {
        "Medium".to_string()
    } else {
        "Normal".to_string()
    };

    (reward, difficulty)
}

pub async fn scan_multiple_repos(
    repos: HashMap<&str, &str>,
    github_token: &str,
) -> Result<Vec<BountyLead>> {
    let mut all_bounties = Vec::new();

    for (owner, repo) in repos {
        match scan_bounties(owner, repo, github_token).await {
            Ok(mut bounties) => {
                all_bounties.append(&mut bounties);
            }
            Err(e) => {
                eprintln!("Warning: Failed to scan {}/{}: {}", owner, repo, e);
            }
        }
    }

    // Remove duplicates and re-sort
    all_bounties.sort_by(|a, b| b.score().cmp(&a.score()));
    all_bounties.dedup_by(|a, b| a.number == b.number && a.repository == b.repository);

    Ok(all_bounties)
}
