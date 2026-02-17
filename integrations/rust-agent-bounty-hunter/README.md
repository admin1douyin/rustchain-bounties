# Rust Agent Bounty Hunter Framework

A Rust-based AI agent framework for discovering, analyzing, and submitting bounty solutions on GitHub repositories.

## Overview

This framework provides a complete automated workflow for bounty hunting:

- **Scanner** - Discovers bounty opportunities across multiple repositories
- **Analyzer** - Evaluates technical complexity and effort estimates
- **Generator** - Creates standardized claim and submission templates
- **Quality** - Validates submissions meet quality standards
- **Submitter** - Handles PR creation and issue updates

## Quick Start

### Prerequisites

- Rust 1.70+
- GitHub Personal Access Token

### Installation

```bash
cd integrations/rust-agent-bounty-hunter
cargo build --release
```

### Configuration

Set your GitHub token:
```bash
export GITHUB_TOKEN="ghp_your_token_here"
```

Or use the `--token` flag with each command.

## Usage

### Scan for Bounties

```bash
cargo run -- scan --owner rustchain --repo rustchain
```

Options:
- `--top N` - Return top N results (default: 20)
- `--output file.json` - Save results to JSON file

### Analyze a Specific Issue

```bash
cargo run -- analyze --owner rustchain --repo rustchain --issue 123
```

### Claim a Bounty

```bash
cargo run -- claim \
  --issue 123 \
  --repo rustchain/rustchain \
  --wallet your_wallet_address \
  --handle your_github_handle
```

Use `--post` to actually submit the claim.

### Submit Completed Work

```bash
cargo run -- submit \
  --issue 123 \
  --repo rustchain/rustchain \
  --pr https://github.com/rustchain/rustchain/pull/456 \
  --wallet your_wallet_address \
  --handle your_github_handle \
  --summary "Fixed critical bug in matches_pattern()" \
  --post
```

### Validate PR Quality

```bash
cargo run -- validate --repo rustchain/rustchain --pr 456
```

### Full Auto Mode

```bash
cargo run -- auto \
  --owner rustchain \
  --repo rustchain \
  --wallet your_wallet_address \
  --handle your_github_handle
```

Use `--dry-run` first to see what would happen without making changes.

## Project Structure

```
integrations/rust-agent-bounty-hunter/
├── Cargo.toml
├── README.md
└── src/
    ├── main.rs           # CLI entry point
    ├── scanner.rs        # Bounty discovery
    ├── analyzer.rs       # Complexity analysis
    ├── generator.rs      # Template generation
    ├── quality.rs        # Quality validation
    └── submitter.rs      # GitHub API interactions
```

## Features

### Multi-Repository Scanning
Scan multiple repositories simultaneously and aggregate results.

### Intelligent Scoring
Bounties are scored based on:
- Reward amount
- Difficulty level
- Complexity of requirements

### Automated Templates
Generate standardized:
- Claim comments
- PR descriptions
- Submission updates

### Quality Checks
Validate submissions against:
- PR description completeness
- Test coverage
- Documentation updates
- Merge readiness

## Bounty Workflow

```
┌─────────────────────────────────────────────────────────────┐
│                    BOUNTY HUNTING FLOW                       │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  1. SCAN    →  2. ANALYZE  →  3. CLAIM  →  4. IMPLEMENT     │
│     🔍            📊              📝           💻            │
│                                                             │
│                 7. GET PAID ←  6. REVIEW  ←  5. SUBMIT       │
│                       💰              ✅           📤        │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Integration

This framework integrates with the RustChain bounty system:

- Compatible with the existing Python `agent_bounty_hunter.py`
- Uses same GitHub labels and workflow
- Outputs compatible claim/submission formats

## Contributing

1. Fork this repository
2. Create a feature branch
3. Add your improvements
4. Submit a PR

## License

MIT License - See repository root for details.
