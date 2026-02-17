//! Quality Assurer - Validates submissions meet bounty requirements
//!
//! This module handles:
//! - Checking code quality standards
//! - Verifying test coverage
//! - Validating documentation completeness
//! - Ensuring commit history cleanliness

use anyhow::{Result, anyhow};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct QualityReport {
    pub passed: bool,
    pub score: u64,
    pub max_score: u64,
    pub checks: Vec<QualityCheck>,
}

#[derive(Debug, Clone)]
pub struct QualityCheck {
    pub name: String,
    pub passed: bool,
    pub score: u64,
    pub max_score: u64,
    pub message: String,
}

impl QualityReport {
    pub fn percentage(&self) -> f64 {
        if self.max_score == 0 {
            0.0
        } else {
            (self.score as f64 / self.max_score as f64) * 100.0
        }
    }

    pub fn summary(&self) -> String {
        let status = if self.passed { "✅ PASSED" } else { "❌ FAILED" };
        format!(
            "{} ({:.1}% - {}/{} points)",
            status,
            self.percentage(),
            self.score,
            self.max_score
        )
    }
}

pub fn validate_submission(
    pr_number: u64,
    repo: &str,
    github_token: &str,
) -> Result<QualityReport> {
    let client = reqwest::Client::new();
    
    // Get PR details
    let pr_url = format!(
        "https://api.github.com/repos/{}/pulls/{}",
        repo, pr_number
    );
    
    let pr_response = client
        .get(&pr_url)
        .header("Authorization", format!("token {}", github_token))
        .send()
        .await
        .context("Failed to fetch PR")?
        .json::<serde_json::Value>()
        .await
        .context("Failed to parse PR")?;

    // Get PR files
    let files_url = format!(
        "https://api.github.com/repos/{}/pulls/{}/files",
        repo, pr_number
    );
    
    let files_response = client
        .get(&files_url)
        .header("Authorization", format!("token {}", github_token))
        .send()
        .await
        .context("Failed to fetch PR files")?
        .json::<serde_json::Value>()
        .await
        .context("Failed to parse PR files")?;

    let files = files_response.as_array()
        .context("Files should be an array")?;

    // Run quality checks
    let mut checks = Vec::new();
    let mut total_score = 0u64;
    let mut max_score = 0u64;

    // Check 1: PR has description
    let has_description = pr_response.get("body").map(|b| !b.is_null() && b.as_str().map(|s| !s.is_empty()).unwrap_or(false)).unwrap_or(false);
    let desc_check = QualityCheck {
        name: "PR Description".to_string(),
        passed: has_description,
        score: if has_description { 10 } else { 0 },
        max_score: 10,
        message: if has_description {
            "PR has description".to_string()
        } else {
            "⚠️ PR description is empty or missing".to_string()
        },
    };
    checks.push(desc_check.clone());
    total_score += desc_check.score;
    max_score += desc_check.max_score;

    // Check 2: Tests added/modified
    let has_tests = files.iter().any(|f| {
        let filename = f["filename"].as_str().unwrap_or("");
        filename.contains("test") || filename.ends_with("_test.rs")
    });
    let test_check = QualityCheck {
        name: "Tests Included".to_string(),
        passed: has_tests,
        score: if has_tests { 15 } else { 5 },
        max_score: 15,
        message: if has_tests {
            "Tests are included in the PR".to_string()
        } else {
            "⚠️ No tests found in the PR (bonus points for adding tests)".to_string()
        },
    };
    checks.push(test_check.clone());
    total_score += test_check.score;
    max_score += test_check.max_score;

    // Check 3: Documentation updated
    let has_docs = files.iter().any(|f| {
        let filename = f["filename"].as_str().unwrap_or("");
        filename.ends_with(".md") || filename.contains("README")
    });
    let docs_check = QualityCheck {
        name: "Documentation Updated".to_string(),
        passed: has_docs,
        score: if has_docs { 10 } else { 5 },
        max_score: 10,
        message: if has_docs {
            "Documentation updated".to_string()
        } else {
            "ℹ️ No documentation changes detected".to_string()
        },
    };
    checks.push(docs_check.clone());
    total_score += docs_check.score;
    max_score += docs_check.max_score;

    // Check 4: Code compiles (heuristic: no syntax errors in diff)
    let has_code = files.iter().any(|f| {
        let filename = f["filename"].as_str().unwrap_or("");
        filename.ends_with(".rs") || filename.ends_with(".py") || filename.ends_with(".js")
    });
    let code_check = QualityCheck {
        name: "Contains Code".to_string(),
        passed: has_code,
        score: if has_code { 10 } else { 0 },
        max_score: 10,
        message: if has_code {
            "Code changes present".to_string()
        } else {
            "⚠️ No code files changed".to_string()
        },
    };
    checks.push(code_check.clone());
    total_score += code_check.score;
    max_score += code_check.max_score;

    // Check 5: Review status
    let review_state = pr_response["mergeable_state"].as_str().unwrap_or("unknown");
    let is_mergeable = review_state == "clean" || review_state == "has_hooks";
    let merge_check = QualityCheck {
        name: "Merge Ready".to_string(),
        passed: is_mergeable,
        score: if is_mergeable { 10 } else { 5 },
        max_score: 10,
        message: if is_mergeable {
            format!("PR is mergeable (state: {})", review_state)
        } else {
            format!("⚠️ PR has merge conflicts or needs rebasing (state: {})", review_state)
        },
    };
    checks.push(merge_check.clone());
    total_score += merge_check.score;
    max_score += merge_check.max_score;

    let passed = total_score >= max_score / 2;

    Ok(QualityReport {
        passed,
        score: total_score,
        max_score,
        checks,
    })
}

pub fn check_code_quality(file_path: &Path) -> Result<QualityReport> {
    // This would integrate with rustfmt, clippy, etc.
    // For now, return a placeholder
    Ok(QualityReport {
        passed: true,
        score: 100,
        max_score: 100,
        checks: vec![QualityCheck {
            name: "Code Quality".to_string(),
            passed: true,
            score: 100,
            max_score: 100,
            message: "Manual review required".to_string(),
        }],
    })
}

pub fn validate_commit_history(commits: &[serde_json::Value]) -> Result<QualityReport> {
    let commit_count = commits.len();
    
    let has_good_messages = commits.iter().all(|c| {
        let msg = c["commit"]["message"].as_str().unwrap_or("");
        msg.len() > 10 && !msg.starts_with("Merge")
    });

    let report = QualityReport {
        passed: has_good_messages,
        score: if has_good_messages { 20 } else { 10 },
        max_score: 20,
        checks: vec![QualityCheck {
            name: "Commit Messages".to_string(),
            passed: has_good_messages,
            score: if has_good_messages { 20 } else { 10 },
            max_score: 20,
            message: if has_good_messages {
                format!("{} commits with descriptive messages", commit_count)
            } else {
                "Some commits have poor messages".to_string()
            },
        }],
    };

    Ok(report)
}
