//! Rust Agent Bounty Hunter - Main Entry Point
//!
//! A Rust-based AI agent framework for discovering, analyzing, and submitting
//! bounty solutions on GitHub repositories.
//!
//! ## Features
//! - Multi-repo bounty scanning
//! - Intelligent complexity analysis
//! - Automated PR description generation
//! - Quality assurance checks
//! - Claim and submission automation
//!
//! ## Usage
//!
//! ```bash
//! # Scan bounties from rustchain repo
//! cargo run -- scan --owner rustchain --repo rustchain --token $GITHUB_TOKEN
//!
//! # Analyze a specific issue
//! cargo run -- analyze --owner rustchain --repo rustchain --issue 123 --token $GITHUB_TOKEN
//!
//! # Generate claim template
//! cargo run -- claim --issue 123 --wallet your_wallet --handle your_handle --token $GITHUB_TOKEN
//!
//! # Submit completion
//! cargo run -- submit --issue 123 --pr https://github.com/repo/pull/456 --token $GITHUB_TOKEN
//! ```

mod scanner;
mod analyzer;
mod generator;
mod quality;
mod submitter;

use crate::scanner::{scan_bounties, scan_multiple_repos};
use crate::analyzer::analyze_bounty;
use crate::generator::{generate_claim_comment, generate_submission_comment};
use crate::quality::validate_submission;
use crate::submitter::{claim_bounty, submit_bounty_completion};
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::env;

const VERSION: &str = "1.0.0";

/// Rust-based AI Agent Bounty Hunter Framework
#[derive(Parser, Debug)]
#[command(name = "bounty-hunter")]
#[command(author = "RustChain Bounty System")]
#[command(version = VERSION)]
#[command(about = "Discover, analyze, and submit bounty solutions", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,

    /// GitHub Authentication Token
    #[arg(short, long, env = "GITHUB_TOKEN")]
    token: Option<String>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Scan repositories for bounty opportunities
    Scan {
        /// Repository owner
        #[arg(short, long)]
        owner: Option<String>,
        /// Repository name
        #[arg(short, long)]
        repo: Option<String>,
        /// Output results to file
        #[arg(short, long)]
        output: Option<String>,
        /// Maximum number of results
        #[arg(short, long, default_value = "20")]
        top: u64,
    },

    /// Analyze a specific bounty issue
    Analyze {
        /// Repository owner
        #[arg(short, long)]
        owner: String,
        /// Repository name
        #[arg(short, long)]
        repo: String,
        /// Issue number
        #[arg(short, long)]
        issue: u64,
    },

    /// Generate a bounty claim template
    Claim {
        /// Issue number
        #[arg(short, long)]
        issue: u64,
        /// Repository (owner/repo format)
        #[arg(short, long)]
        repo: String,
        /// Wallet address for reward
        #[arg(short, long)]
        wallet: String,
        /// GitHub handle
        #[arg(short, long)]
        handle: String,
        /// Actually post the claim (dry-run by default)
        #[arg(short, long)]
        post: bool,
    },

    /// Submit completed bounty solution
    Submit {
        /// Issue number
        #[arg(short, long)]
        issue: u64,
        /// Repository (owner/repo format)
        #[arg(short, long)]
        repo: String,
        /// PR URL
        #[arg(short, long)]
        pr: String,
        /// Wallet address for reward
        #[arg(short, long)]
        wallet: String,
        /// GitHub handle
        #[arg(short, long)]
        handle: String,
        /// Summary of changes
        #[arg(short, long)]
        summary: String,
        /// Actually submit (dry-run by default)
        #[arg(short, long)]
        post: bool,
    },

    /// Check submission quality
    Validate {
        /// Repository (owner/repo format)
        #[arg(short, long)]
        repo: String,
        /// PR number
        #[arg(short, long)]
        pr: u64,
    },

    /// Full workflow: scan, analyze, claim, implement, submit
    Auto {
        /// Repository owner
        #[arg(short, long)]
        owner: String,
        /// Repository name
        #[arg(short, long)]
        repo: String,
        /// Wallet address
        #[arg(short, long)]
        wallet: String,
        /// GitHub handle
        #[arg(short, long)]
        handle: String,
        /// Automatically submit PRs (highly recommended: use --dry-run first)
        #[arg(long)]
        dry_run: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    // Get token from args or env
    let token = match args.token {
        Some(t) => t,
        None => env::var("GITHUB_TOKEN").context("GITHUB_TOKEN not set. Use --token or set GITHUB_TOKEN env var.")?
    };

    match args.command {
        Commands::Scan { owner, repo, output, top } => {
            println!("🔍 Scanning for bounties...");
            
            let bounties = if let (Some(o), Some(r)) = (owner, repo) {
                scan_bounties(&o, &r, &token).await?
            } else {
                // Default repos
                let mut repos = HashMap::new();
                repos.insert("Scottcjn", "rustchain-bounties");
                repos.insert("rustchain", "rustchain");
                scan_multiple_repos(repos, &token).await?
            };

            println!("\n📊 Found {} bounty opportunities:", bounties.len());
            for (i, bounty in bounties.iter().take(top as usize).enumerate() {
                println!("\n{}. #{} - {}", i + 1, bounty.number, bounty.title);
                println!("   Repository: {}", bounty.repository);
                println!("   Reward: {} | Difficulty: {}", bounty.reward_estimate, bounty.difficulty);
                println!("   URL: {}", bounty.url);
            }

            if let Some(path) = output {
                let json = serde_json::to_string_pretty(&bounties)?;
                std::fs::write(&path, json)?;
                println!("\n✅ Results saved to {}", path);
            }
        }

        Commands::Analyze { owner, repo, issue } => {
            println!("📊 Analyzing issue #{issue}...");

            let client = reqwest::Client::new();
            let url = format!(
                "https://api.github.com/repos/{}/{}/issues/{}",
                owner, repo, issue
            );

            let issue_data: serde_json::Value = client
                .get(&url)
                .header("Authorization", format!("token {}", token))
                .send()
                .await
                .context("Failed to fetch issue")?
                .json()
                .await
                .context("Failed to parse issue")?;

            let title = issue_data["title"].as_str().unwrap_or("");
            let body = issue_data["body"].as_str().unwrap_or("");

            let analysis = analyze_bounty(issue, title, body);

            println!("\n📋 Analysis for Issue #{}", analysis.number);
            println!("   Title: {}", analysis.title);
            println!("   Complexity: {}", analysis.technical_complexity.as_str());
            println!("   Estimated Effort: {}", analysis.estimated_effort);
            println!("\n💡 Implementation Notes:\n{}", analysis.implementation_notes);
            
            if !analysis.risks.is_empty() {
                println!("\n⚠️  Risks:");
                for risk in &analysis.risks {
                    println!("   - {}", risk);
                }
            }
        }

        Commands::Claim { issue, repo, wallet, handle, post } => {
            println!("📝 Generating claim template for issue #{issue}...");

            let parts: Vec<&str> = repo.split('/').collect();
            let (owner, repo_name) = (parts[0], parts[1]);

            let client = reqwest::Client::new();
            let url = format!(
                "https://api.github.com/repos/{}/{}/issues/{}",
                owner, repo_name, issue
            );

            let issue_data: serde_json::Value = client
                .get(&url)
                .header("Authorization", format!("token {}", token))
                .send()
                .await
                .context("Failed to fetch issue")?
                .json()
                .await
                .context("Failed to parse issue")?;

            let title = issue_data["title"].as_str().unwrap_or("");
            let body = issue_data["body"].as_str().unwrap_or("");

            let analysis = analyze_bounty(issue, title, body);
            let template = generate_claim_comment(issue, &repo, &wallet, &handle, &analysis);

            if post {
                let result = claim_bounty(owner, repo_name, issue, &template.body, "claimed", &token).await?;
                println!("\n✅ {}", result.message);
                if let Some(url) = result.url {
                    println!("   URL: {}", url);
                }
            } else {
                println!("\n📝 Claim Template (--post to submit):");
                println!("{}", template.body);
            }
        }

        Commands::Submit { issue, repo, pr, wallet, handle, summary, post } => {
            println!("📤 Generating submission for issue #{issue}...");

            let parts: Vec<&str> = repo.split('/').collect();
            let (owner, repo_name) = (parts[0], parts[1]);

            let template = generate_submission_comment(issue, &repo, &pr, &wallet, &handle, &summary);

            if post {
                let result = submit_bounty_completion(owner, repo_name, issue, &pr, &template.body, &token).await?;
                println!("\n✅ {}", result.message);
            } else {
                println!("\n📝 Submission Template (--post to submit):");
                println!("{}", template.body);
            }
        }

        Commands::Validate { repo, pr } => {
            println!("🔍 Validating PR #{}...", pr);

            let parts: Vec<&str> = repo.split('/').collect();
            let (owner, repo_name) = (parts[0], parts[1]);

            let report = validate_submission(pr, &repo, &token).await?;
            
            println!("\n📊 Quality Report for PR #{}", pr);
            println!("   {}", report.summary());
            println!("\nDetails:");
            for check in &report.checks {
                let status = if check.passed { "✅" } else { "❌" };
                println!("   {} {}: {} ({} pts)", status, check.name, check.message, check.score);
            }
        }

        Commands::Auto { owner, repo, wallet, handle, dry_run } => {
            println!("🚀 Starting auto bounty hunter...");
            println!("   Target: {}/{}", owner, repo);
            println!("   Mode: {}", if dry_run { "DRY RUN" } else { "LIVE" });
            
            if dry_run {
                println!("\n⚠️  Dry run mode - no actual changes will be made\n");
            }

            // Step 1: Scan
            println!("\n1️⃣  Scanning for bounties...");
            let bounties = scan_bounties(&owner, &repo, &token).await?;
            println!("   Found {} bounties", bounties.len());

            // Step 2: Pick top bounty
            if let Some(first_bounty) = bounties.first() {
                println!("\n2️⃣  Selecting top bounty: #{} - {}", first_bounty.number, first_bounty.title);
                
                // Analyze
                println!("3️⃣  Analyzing...");
                let analysis = analyze_bounty(first_bounty.number, &first_bounty.title, &first_bounty.body);
                println!("   Complexity: {}", analysis.technical_complexity.as_str());
                println!("   Effort: {}", analysis.estimated_effort);

                // Generate claim
                println!("4️⃣  Generating claim...");
                let claim = generate_claim_comment(
                    first_bounty.number, 
                    &format!("{}/{}", owner, repo), 
                    &wallet, 
                    &handle, 
                    &analysis
                );

                if dry_run {
                    println!("\n📝 Would claim bounty #{}", first_bounty.number);
                    println!("{}", claim.body);
                } else {
                    let result = claim_bounty(
                        &owner, &repo, first_bounty.number, &claim.body, "claimed", &token
                    ).await?;
                    println!("\n✅ Claim submitted: {}", result.message);
                }
            } else {
                println!("\n⚠️  No bounties found to claim");
            }
        }
    }

    Ok(())
}
