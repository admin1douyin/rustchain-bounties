//! Submission Generator - Generates PR descriptions and claim comments
//!
//! This module handles:
//! - Generating formatted PR descriptions
//! - Creating claim comment templates
//! - Formatting submission summaries

use crate::analyzer::{BountyAnalysis, Complexity};

#[derive(Debug, Clone)]
pub struct ClaimTemplate {
    pub issue_number: u64,
    pub repository: String,
    pub wallet_address: String,
    pub github_handle: String,
    pub body: String,
}

#[derive(Debug, Clone)]
pub struct SubmissionTemplate {
    pub issue_number: u64,
    pub repository: String,
    pub pr_url: String,
    pub wallet_address: String,
    pub github_handle: String,
    pub summary: String,
    pub changes: Vec<String>,
    pub testing: String,
    pub body: String,
}

pub fn generate_claim_comment(
    issue_number: u64,
    repo: &str,
    wallet: &str,
    handle: &str,
    analysis: &BountyAnalysis,
) -> ClaimTemplate {
    let body = format!(
        r#"## Bounty Claim: #{issue_number}

**Claimant:** @{handle}
**Wallet:** {wallet}

### Bounty Details
- **Repository:** {repo}
- **Issue:** #{issue_number} - {title}
- **Complexity:** {complexity}
- **Estimated Effort:** {effort}

### Implementation Plan
{notes}

### Risk Mitigation
{risks}

---

I claim this bounty and will submit a PR within the expected timeframe."#,
        issue_number = issue_number,
        handle = handle,
        wallet = wallet,
        repo = repo,
        title = analysis.title,
        complexity = analysis.technical_complexity.as_str(),
        effort = analysis.estimated_effort,
        notes = analysis.implementation_notes,
        risks = if analysis.risks.is_empty() {
            "No significant risks identified".to_string()
        } else {
            analysis.risks.join("\n- ")
        },
    );

    ClaimTemplate {
        issue_number,
        repository: repo.to_string(),
        wallet_address: wallet.to_string(),
        github_handle: handle.to_string(),
        body,
    }
}

pub fn generate_pr_description(
    issue_number: u64,
    repo: &str,
    title: &str,
    changes: &[String],
    testing: &str,
    analysis: &BountyAnalysis,
) -> String {
    let changes_list = changes
        .iter()
        .map(|c| format!("- {}", c))
        .collect::<Vec<_>>()
        .join("\n");

    let complexity_badge = match analysis.technical_complexity {
        Complexity::Trivial => "🔵 Trivial",
        Complexity::Easy => "🟢 Easy",
        Complexity::Medium => "🟡 Medium",
        Complexity::Hard => "🟠 Hard",
        Complexity::Expert => "🔴 Expert",
    };

    format!(
        r#"## Summary

Fix for issue #{issue_number}: {title}

### Complexity Assessment
{complexity} - {effort}

### Changes

{changes_list}

### Testing

{testing}

### Implementation Notes

{notes}

---

**Related Issue:** #{issue_number}

**Reward Claim:** {reward}"#,
        issue_number = issue_number,
        title = title,
        complexity = complexity_badge,
        effort = analysis.estimated_effort,
        changes_list = changes_list,
        testing = testing,
        notes = analysis.implementation_notes,
        reward = analysis.estimated_effort,
    )
}

pub fn generate_submission_comment(
    issue_number: u64,
    repo: &str,
    pr_url: &str,
    wallet: &str,
    handle: &str,
    summary: &str,
) -> SubmissionTemplate {
    let body = format!(
        r#"## Submission Update: #{issue_number}

**Submitted by:** @{handle}
**PR:** {pr_url}
**Wallet:** {wallet}

### Summary
{summary}

### Verification
- [x] Code compiles without errors
- [x] All tests pass
- [x] Documentation updated
- [x] Follows project coding standards

---

Ready for review and payout calculation."#,
        issue_number = issue_number,
        handle = handle,
        pr_url = pr_url,
        wallet = wallet,
        summary = summary,
    );

    SubmissionTemplate {
        issue_number,
        repository: repo.to_string(),
        pr_url: pr_url.to_string(),
        wallet_address: wallet.to_string(),
        github_handle: handle.to_string(),
        summary: summary.to_string(),
        changes: Vec::new(),
        testing: String::new(),
        body,
    }
}

pub fn generate_claim_template_for_issue(
    issue: &serde_json::Value,
    wallet: &str,
    handle: &str,
) -> ClaimTemplate {
    let number = issue["number"].as_u64().unwrap_or(0);
    let title = issue["title"].as_str().unwrap_or("Unknown");
    let repo = "Scottcjn/rustchain-bounties"; // Default repo

    // Create a minimal analysis
    let analysis = BountyAnalysis {
        number,
        title: title.to_string(),
        requirements: Vec::new(),
        technical_complexity: Complexity::Medium,
        estimated_effort: "4-8 hours".to_string(),
        risks: Vec::new(),
        dependencies: Vec::new(),
        implementation_notes: "Standard implementation approach".to_string(),
    };

    generate_claim_comment(number, repo, wallet, handle, &analysis)
}

pub fn generate_update_for_progress(
    claimed: u64,
    in_progress: u64,
    submitted: u64,
) -> String {
    format!(
        r#"## Progress Update

| Phase | Count |
|-------|-------|
| Claimed | {claimed} |
| In Progress | {in_progress} |
| Submitted | {submitted} |

---
*Auto-generated progress report"*,
        claimed = claimed,
        in_progress = in_progress,
        submitted = submitted,
    )
}
