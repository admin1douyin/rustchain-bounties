# RustChain Protocol Documentation

**Generated**: 2026-02-16
**Version**: 2.2.1-rip200

---

## Overview

RustChain is a **Proof-of-Antiquity (PoA)** blockchain that rewards real vintage hardware with higher mining multipliers than modern machines.

### Key Features
- 🎯 **Hardware Diversity**: Rewards vintage hardware (PowerPC G4/G5, 68K Macs, SPARC)
- 🔐 **Hardware Fingerprinting**: 6+1 checks prevent VMs/emulators
- 📊 **Live Network**: Active at https://50.28.86.131

---

## RIP-200 Proof-of-Attestation Consensus

RIP-200 is RustChain's consensus mechanism that validates miner hardware authenticity.

### Epoch Structure
```
┌─────────────────────────────────────────┐
│              EPOCH                       │
│  ┌───────────┐  ┌───────────┐           │
│  │ Attestation│  │ Settlement│           │
│  │   Phase   │  │   Phase   │           │
│  └───────────┘  └───────────┘           │
└─────────────────────────────────────────┘
```

### Attestation Flow
1. Miner submits hardware fingerprint
2. Node validates 6+1 fingerprint checks
3. Attestation score calculated
4. Rewards distributed based on antiquity multiplier

---

## Hardware Fingerprinting (6+1 Checks)

| Check | Description | Impact |
|-------|-------------|--------|
| 1 | Cache timing profile | Validates CPU type |
| 2 | SIMD characteristics | Distinguishes G4/G5 from x86 |
| 3 | Thermal drift patterns | Vintage vs modern silicon |
| 4 | Entropy score | Hardware RNG quality |
| 5 | Device architecture | Claims vs fingerprints |
| 6 | Clock drift | Timing consistency |
| +1 | Manual review | Edge cases |

### Antiquity Multipliers

| Hardware | Multiplier |
|----------|------------|
| PowerPC G4 | 2.5x |
| PowerPC G5 | 2.0x |
| PowerPC (Vintage) | 2.0x |
| Apple Silicon (Modern) | 1.2x |
| x86-64 (Modern) | 1.0x |

---

## Token Economics

- **Native Token**: RTC (RustChain Token)
- **Supply**: Fixed at creation
- **Distribution**: 
  - Mining rewards (70%)
  - Attestation nodes (20%)
  - Treasury (10%)

---

## API Reference

### Base URL
```
https://50.28.86.131
```

### Endpoints

#### Health Check
```bash
curl -sk https://50.28.86.131/health
```

**Response:**
```json
{
  "ok": true,
  "version": "2.2.1-rip200",
  "uptime_s": 49017,
  "backup_age_hours": 7.69,
  "tip_age_slots": 0,
  "db_rw": true
}
```

#### Get Miners
```bash
curl -sk https://50.28.86.131/api/miners
```

---

## Network Architecture

```
┌─────────────────────────────────────────────────────┐
│                    RustChain Network                 │
│  ┌─────────────┐     ┌─────────────┐                │
│  │   Attestation│◄────│   Miners    │                │
│  │    Nodes    │     │   (9 active)│                │
│  └──────┬──────┘     └─────────────┘                │
│         │                                          │
│         ▼                                          │
│  ┌─────────────────────────────────────┐           │
│  │         Ergo Anchoring               │           │
│  └─────────────────────────────────────┘           │
└─────────────────────────────────────────────────────┘
```

---

## Resources

- **Block Explorer**: https://50.28.86.131/explorer
- **GitHub Repo**: https://github.com/Scottcjn/Rustchain
- **Bounty Board**: https://github.com/Scottcjn/rustchain-bounties
