//! Bounty Analyzer - Analyzes bounty requirements and technical complexity
//!
//! This module handles:
//! - Extracting technical requirements from issue descriptions
//! - Estimating implementation effort
//! - Identifying dependencies and risks

use anyhow::Result;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct BountyAnalysis {
    pub number: u64,
    pub title: String,
    pub requirements: Vec<String>,
    pub technical_complexity: Complexity,
    pub estimated_effort: String,
    pub risks: Vec<String>,
    pub dependencies: Vec<String>,
    pub implementation_notes: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Complexity {
    Trivial,
    Easy,
    Medium,
    Hard,
    Expert,
}

impl Complexity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Complexity::Trivial => "🔵 Trivial (< 1 hour)",
            Complexity::Easy => "🟢 Easy (1-4 hours)",
            Complexity::Medium => "🟡 Medium (4-8 hours)",
            Complexity::Hard => "🟠 Hard (8-16 hours)",
            Complexity::Expert => "🔴 Expert (16+ hours)",
        }
    }
}

pub fn analyze_bounty(
    number: u64,
    title: &str,
    body: &str,
) -> BountyAnalysis {
    let requirements = extract_requirements(body);
    let complexity = assess_complexity(&requirements, title);
    let effort = estimate_effort(&complexity);
    let risks = identify_risks(body, &requirements);
    let dependencies = find_dependencies(body);
    let notes = generate_implementation_notes(title, body, &complexity);

    BountyAnalysis {
        number,
        title: title.to_string(),
        requirements,
        technical_complexity: complexity,
        estimated_effort: effort,
        risks,
        dependencies,
        implementation_notes: notes,
    }
}

fn extract_requirements(body: &str) -> Vec<String> {
    let mut requirements = Vec::new();
    let lines: Vec<&str> = body.lines().collect();
    
    for line in lines {
        let line = line.trim().to_lowercase();
        
        // Look for requirement indicators
        if line.starts_with("- ") || line.starts_with("* ") || line.starts_with("1.") {
            let req = line.trim_start_matches("- ")
                .trim_start_matches("* ")
                .trim_start_matches("1. ")
                .to_string();
            if !req.is_empty() && req.len() > 3 {
                requirements.push(req);
            }
        }
        
        // Look for "should", "must", "need to" patterns
        if line.contains("should") || line.contains("must") || line.contains("need to") {
            requirements.push(line.to_string());
        }
    }

    requirements
}

fn assess_complexity(requirements: &[String], title: &str) -> Complexity {
    let title_lower = title.to_lowercase();
    let req_count = requirements.len();
    
    // Check for complexity indicators
    if title_lower.contains("security") || title_lower.contains("critical") {
        return Complexity::Expert;
    }
    
    if title_lower.contains("refactor") || title_lower.contains("architecture") {
        return Complexity::Hard;
    }
    
    if title_lower.contains("test") || title_lower.contains("documentation") {
        return Complexity::Trivial;
    }
    
    // Count complex keywords
    let complex_keywords = ["async", "concurrent", "distributed", "consensus", "crypto"];
    let hard_keywords = ["database", "api", "integration", "performance"];
    let easy_keywords = ["fix", "update", "minor", "simple"];
    
    let mut score = 0;
    for req in requirements {
        for kw in &complex_keywords {
            if req.contains(kw) {
                score += 3;
            }
        }
        for kw in &hard_keywords {
            if req.contains(kw) {
                score += 2;
            }
        }
        for kw in &easy_keywords {
            if req.contains(kw) {
                score -= 1;
            }
        }
    }
    
    // Factor in requirement count
    let count_factor = match req_count {
        0..=2 => -1,
        3..=5 => 0,
        6..=10 => 1,
        _ => 2,
    };
    score += count_factor;

    match score {
        s if s >= 8 => Complexity::Expert,
        s if s >= 5 => Complexity::Hard,
        s if s >= 3 => Complexity::Medium,
        s if s >= 1 => Complexity::Easy,
        _ => Complexity::Trivial,
    }
}

fn estimate_effort(complexity: &Complexity) -> String {
    match complexity {
        Complexity::Trivial => "< 1 hour".to_string(),
        Complexity::Easy => "1-4 hours".to_string(),
        Complexity::Medium => "4-8 hours".to_string(),
        Complexity::Hard => "8-16 hours".to_string(),
        Complexity::Expert => "16+ hours".to_string(),
    }
}

fn identify_risks(body: &str, requirements: &[String]) -> Vec<String> {
    let mut risks = Vec::new();
    
    // Check for breaking change indicators
    if body.to_lowercase().contains("breaking") {
        risks.push("Breaking change - requires migration guide".to_string());
    }
    
    // Check for production impact
    if body.to_lowercase().contains("production") || body.to_lowercase().contains("live") {
        risks.push("Production impact - requires thorough testing".to_string());
    }
    
    // Check for security implications
    if body.to_lowercase().contains("security") || body.to_lowercase().contains("vulnerability") {
        risks.push("Security-sensitive - requires security review".to_string());
    }
    
    // Check for external dependencies
    if body.to_lowercase().contains("api") || body.to_lowercase().contains("external") {
        risks.push("External dependency - may break if API changes".to_string());
    }
    
    // Complexity risks
    if requirements.len() > 10 {
        risks.push("Many requirements - risk of scope creep".to_string());
    }
    
    risks
}

fn find_dependencies(body: &str) -> Vec<String> {
    let mut deps = Vec::new();
    let body_lower = body.to_lowercase();
    
    if body_lower.contains("tokio") || body_lower.contains("async") {
        deps.push("tokio async runtime".to_string());
    }
    if body_lower.contains("serde") {
        deps.push("serde serialization".to_string());
    }
    if body_lower.contains("database") || body_lower.contains("sql") {
        deps.push("database backend".to_string());
    }
    if body_lower.contains("api") || body_lower.contains("http") {
        deps.push("HTTP client library".to_string());
    }
    
    deps
}

fn generate_implementation_notes(title: &str, body: &str, complexity: &Complexity) -> String {
    let mut notes = String::new();
    
    match complexity {
        Complexity::Expert | Complexity::Hard => {
            notes.push_str("⚠️ Complex implementation - consider phased approach\n");
            notes.push_str("- Break into smaller PRs if possible\n");
            notes.push_str("- Add comprehensive tests\n");
        }
        Complexity::Medium => {
            notes.push_str("- Standard implementation approach\n");
            notes.push_str("- Add unit tests for edge cases\n");
        }
        _ => {
            notes.push_str("- Straightforward fix\n");
            notes.push_str("- Quick turnaround expected\n");
        }
    }
    
    // Add architecture hint
    if title.to_lowercase().contains("refactor") {
        notes.push_str("- Follow existing code patterns\n");
        notes.push_str("- Preserve existing behavior\n");
    }
    
    notes.trim().to_string()
}
