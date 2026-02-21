# Secure LightClaw Development Plan

## Project Vision

Secure LightClaw is a secure, lightweight AI assistant framework inspired by OpenClaw-style ecosystems while prioritizing performance, modularity, and safety. The framework should:

- Run efficiently on low-power devices and embedded targets.
- Use strong security boundaries by default.
- Support multiple channels (CLI, web, and messaging adapters).
- Offer a modular plugin/skill architecture.
- Include auditability and policy-driven permission controls.
- Scale from edge devices to server and cluster deployments.

## High-Level Capability Targets

| Category | Capability |
| --- | --- |
| Core Engine | Fast, modular assistant loop |
| Security | Sandboxed tools/plugins and policy controls |
| Memory | Persistent context with safe storage access |
| Integrations | Multi-channel interfaces (CLI, web, messaging) |
| LLM Providers | Pluggable provider abstraction with failover |
| Extensibility | Skill/plugin ecosystem with manifests |
| Performance | Small footprint and fast startup |
| Deployability | Single-node edge through cloud deployment |

## Inspirations and Design Inputs

- **OpenClaw**: channel integrations, scheduling/heartbeat patterns.
- **PicoClaw**: tiny binaries and low startup latency.
- **ZeptoClaw**: Rust-first safety, policy gating, multi-agent patterns.
- **Nanobot**: minimal code footprint and modular skill composition.

These are design inspirations only; implementation should remain original and security-focused.

## Recommended Technology Stack

| Layer | Preferred Technology |
| --- | --- |
| Core agent runtime | Rust |
| Plugin runtime | WASM-first (native modules optional with stricter controls) |
| API and messaging adapters | REST/webhook adapters |
| Storage | SQLite + encrypted blob support |
| Deployment | Static binary + container image |
| Tool execution | Sandboxed runtime + explicit permission matrix |
| LLM interface | Provider-agnostic abstraction (OpenAI, Anthropic, local, etc.) |

## Core Architecture

### 1) Modular Agent Core

- Unified event/message loop shared by CLI/API/channel adapters.
- Plugin manager for discovery, loading, versioning, and permission checks.
- Policy engine that blocks unsafe operations unless explicitly allowed.
- Provider layer supporting retries, routing, and fallback.

### 2) Security-First Defaults

- **Sandbox execution** for plugins/tools (WASM preferred).
- **Fine-grained permissions** for network, file I/O, secret access, and external calls.
- **Audit logs** for all sensitive actions.
- **Skill vetting path** with deterministic manifests and restricted install sources.

### 3) Persistent Memory and Context

- Structured local memory via SQLite.
- Encryption for sensitive context and credential-linked metadata.
- Memory scopes, tagging, retention/expiry, and policy-governed access.

### 4) Channel Integration Layer

- Unified adapter interface for:
  - CLI
  - Web UI / API
  - Telegram / Discord / WhatsApp
  - Scheduled tasks and webhooks

### 5) Plugin and Skill System

Each plugin should declare:

- Capabilities
- Required permissions
- Input/output contract
- Version and compatibility metadata

Execution should fail closed if manifest validation or policy checks fail.

### 6) LLM Provider Abstraction

- Multiple providers through one interface.
- Cost-aware routing and fallback policies.
- Token budget controls and guardrails.
- Provider health checks and telemetry.

### 7) Deployment Profiles

| Tier | Target |
| --- | --- |
| Edge | IoT and low-cost boards |
| Desktop | macOS / Linux / Windows |
| Cloud | VPS / container deployments |
| HA | Multi-tenant clustered deployments |

## Testing and QA Strategy

- Unit tests for core engine, policies, and adapters.
- Integration tests for provider routing and plugin execution.
- Fuzz testing for parser and API boundary hardening.
- Security tests for sandbox escapes and malformed plugin manifests.
- CI checks with dependency and artifact scanning.

## Phased Roadmap

### Phase 1: Core MVP

- Agent loop
- CLI interface
- Basic provider abstraction
- Plugin skeleton and manifest schema

### Phase 2: Security Hardening

- WASM sandbox runtime
- Policy and permission model
- Audit/event logging

### Phase 3: Multi-Platform

- Messaging adapters
- Web UI/API
- Scheduler and heartbeat jobs

### Phase 4: Ecosystem

- Plugin registry model
- Vetting and publication workflow
- Developer docs and onboarding

### Phase 5: Optimization

- Memory/CPU profiling
- Edge build profiles
- Startup time and binary size targets

## Initial Deliverables

- Architecture and security design documentation.
- Rust project scaffold for modular core runtime.
- Provider abstraction crate and reference adapters.
- Plugin system with signed or vetted manifest support.
- Security test harness and CI baseline.
- Deployment scripts (local and container-based).
