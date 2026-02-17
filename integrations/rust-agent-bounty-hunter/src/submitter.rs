//! Bounty Submitter - Handles PR creation and comment posting
//!
//! This module handles:
//! - Creating pull requests
//! - Posting claim comments
//! - Updating issue status
//! - Managing submission lifecycle

use anyhow::{Result, Context};
use serde_json::json;

#[derive(Debug, Clone)]
pub struct SubmitResult {
    pub success: bool,
    pub action: String,
    pub url: Option<String>,
    pub message: String,
}

pub async fn create_pr(
    owner: &str,
    repo: &str,
    title: &str,
    body: &str,
    head: &str,
    base: &str,
    github_token: &str,
) -> Result<SubmitResult> {
    let client = reqwest::Client::new();
    
    let url = format!(
        "https://api.github.com/repos/{}/{}/pulls",
        owner, repo
    );
    
    let payload = json!({
        "title": title,
        "body": body,
        "head": head,
        "base": base
    });

    let response = client
        .post(&url)
        .header("Authorization", format!("token {}", github_token))
        .header("Accept", "application/vnd.github.v3+json")
        .json(&payload)
        .send()
        .await
        .context("Failed to create PR")?;

    if response.status().is_success() {
        let pr: serde_json::Value = response.json().await.context("Failed to parse PR response")?;
        let pr_url = pr["html_url"].as_str().unwrap_or("").to_string();
        
        Ok(SubmitResult {
            success: true,
            action: "PR Created".to_string(),
            url: Some(pr_url),
            message: format!("PR created successfully: {}", pr_url),
        })
    } else {
        let error = response.text().await.context("Failed to get error")?;
        Ok(SubmitResult {
            success: false,
            action: "PR Creation Failed".to_string(),
            url: None,
            message: format!("Failed to create PR: {}", error),
        })
    }
}

pub async fn post_issue_comment(
    owner: &str,
    repo: &str,
    issue_number: u64,
    comment: &str,
    github_token: &str,
) -> Result<SubmitResult> {
    let client = reqwest::Client::new();
    
    let url = format!(
        "https://api.github.com/repos/{}/{}/issues/{}/comments",
        owner, repo, issue_number
    );
    
    let payload = json!({
        "body": comment
    });

    let response = client
        .post(&url)
        .header("Authorization", format!("token {}", github_token))
        .header("Accept", "application/vnd.github.v3+json")
        .json(&payload)
        .send()
        .await
        .context("Failed to post comment")?;

    if response.status().is_success() {
        let comment_response: serde_json::Value = response.json().await.context("Failed to parse comment response")?;
        let comment_url = comment_response["html_url"].as_str().unwrap_or("").to_string();
        
        Ok(SubmitResult {
            success: true,
            action: "Comment Posted".to_string(),
            url: Some(comment_url),
            message: "Comment posted successfully".to_string(),
        })
    } else {
        let error = response.text().await.context("Failed to get error")?;
        Ok(SubmitResult {
            success: false,
            action: "Comment Failed".to_string(),
            url: None,
            message: format!("Failed to post comment: {}", error),
        })
    }
}

pub async fn update_issue(
    owner: &str,
    repo: &str,
    issue_number: u64,
    labels: Option<Vec<&str>>,
    state: Option<&str>,
    github_token: &str,
) -> Result<SubmitResult> {
    let client = reqwest::Client::new();
    
    let url = format!(
        "https://api.github.com/repos/{}/{}/issues/{}",
        owner, repo, issue_number
    );
    
    let mut payload = json!({});
    
    if let Some(labels) = labels {
        payload["labels"] = json!(labels);
    }
    
    if let Some(state) = state {
        payload["state"] = json!(state);
    }

    let response = client
        .patch(&url)
        .header("Authorization", format!("token {}", github_token))
        .header("Accept", "application/vnd.github.v3+json")
        .json(&payload)
        .send()
        .await
        .context("Failed to update issue")?;

    if response.status().is_success() {
        Ok(SubmitResult {
            success: true,
            action: "Issue Updated".to_string(),
            url: None,
            message: "Issue updated successfully".to_string(),
        })
    } else {
        let error = response.text().await.context("Failed to get error")?;
        Ok(SubmitResult {
            success: false,
            action: "Update Failed".to_string(),
            url: None,
            message: format!("Failed to update issue: {}", error),
        })
    }
}

pub async fn submit_pr_review(
    owner: &str,
    repo: &str,
    pr_number: u64,
    event: &str,
    body: &str,
    github_token: &str,
) -> Result<SubmitResult> {
    let client = reqwest::Client::new();
    
    let url = format!(
        "https://api.github.com/repos/{}/{}/pulls/{}/reviews",
        owner, repo, pr_number
    );
    
    let payload = json!({
        "event": event, // APPROVE, REQUEST_CHANGES, COMMENT
        "body": body
    });

    let response = client
        .post(&url)
        .header("Authorization", format!("token {}", github_token))
        .header("Accept", "application/vnd.github.v3+json")
        .json(&payload)
        .send()
        .await
        .context("Failed to submit review")?;

    if response.status().is_success() {
        Ok(SubmitResult {
            success: true,
            action: "Review Submitted".to_string(),
            url: None,
            message: format!("Review submitted: {}", event),
        })
    } else {
        let error = response.text().await.context("Failed to get error")?;
        Ok(SubmitResult {
            success: false,
            action: "Review Failed".to_string(),
            url: None,
            message: format!("Failed to submit review: {}", error),
        })
    }
}

pub async fn claim_bounty(
    owner: &str,
    repo: &str,
    issue_number: u64,
    claim_comment: &str,
    add_label: &str,
    github_token: &str,
) -> Result<SubmitResult> {
    // Post claim comment
    let comment_result = post_issue_comment(owner, repo, issue_number, claim_comment, github_token).await?;
    
    if !comment_result.success {
        return Ok(comment_result);
    }
    
    // Add claim label
    let label_result = update_issue(
        owner, repo, issue_number,
        Some(vec![add_label, "claimed"]),
        None,
        github_token
    ).await?;
    
    Ok(SubmitResult {
        success: label_result.success,
        action: "Bounty Claimed".to_string(),
        url: comment_result.url,
        message: format!("Bounty claimed. Comment: {}, Label: {}", comment_result.message, label_result.message),
    })
}

pub async fn submit_bounty_completion(
    owner: &str,
    repo: &str,
    issue_number: u64,
    pr_url: &str,
    submission_comment: &str,
    github_token: &str,
) -> Result<SubmitResult> {
    // Post submission comment
    let comment_result = post_issue_comment(owner, repo, issue_number, submission_comment, github_token).await?;
    
    if !comment_result.success {
        return Ok(comment_result);
    }
    
    // Update issue to "submitted" status
    let update_result = update_issue(
        owner, repo, issue_number,
        Some(vec!["submitted", "under-review"]),
        None,
        github_token
    ).await?;
    
    Ok(SubmitResult {
        success: update_result.success,
        action: "Submission Complete".to_string(),
        url: Some(pr_url.to_string()),
        message: format!("Submission complete. PR: {}, Status updated", pr_url),
    })
}
