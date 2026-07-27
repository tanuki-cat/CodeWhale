# Claude Repository Guidance

Read `AGENTS.md` first. This file exists as a compatibility instruction source
for Claude-based agents working in this repository.

## Intent is the artifact

See `AGENTS.md` → "Intent is the artifact" for the full statement. The short
version, because it governs almost every judgment call you will make here:

- Generating code against current `main` is now **cheaper and faster** than
  recovering, rebasing, or reconciling old code. Default to that.
- **Rewriting any part of this project is always in scope**, including the whole
  project. Nothing is load-bearing merely because it exists.
- **Use git; do not be governed by it.** A far-behind branch is a note about
  something we once wanted — not a debt. High conflict count is a signal to
  rewrite, not a task list.
- Stranded work becomes an **issue describing the intent**, plus a deleted
  branch — not a heroic merge.
- The non-negotiable check: **confirm main doesn't already do it.** Grep the
  symbols and behavior, not the commit history. Re-landing landed work is the
  failure mode this ethos creates.
- Limits: `main` stays protected, published history and shipped tags stay
  immutable, contributor credit carries onto the rewrite, and the
  do-not-delete guardrail in `AGENTS.md` still binds.

## Stewardship Defaults

- Treat community PRs and issues as maintainer evidence. Inspect code, tests,
  linked issues, comments, and CI before merging, harvesting, closing, or
  deferring work.
- CodeWhale started as a DeepSeek-only harness; it's now about building the
  greatest possible coding harness with the help of an open-source community.
  Keep CodeWhale branding and every model/provider first-class — none
  privileged — and preserve legacy migration care.
- Preserve contributor credit for harvested work with authorship,
  `Co-authored-by`, `Harvested from PR #N by @handle`, and changelog/release
  notes where applicable. Keep `Co-authored-by` trailers to human contributors,
  using canonical GitHub-noreply identities from `.github/AUTHOR_MAP` — the
  `check-coauthor-trailers.py` CI gate accepts those and rejects bot/tool ones
  (Claude, codex, cursor), so use a plain commit body to note agent assistance.

## Scratch Integration Branches

Applies to **live** queue work. Once a branch has drifted far enough that the
merge is an excavation, stop and apply "Intent is the artifact" instead.

- For release queues, create disposable local branches from the real landing
  branch, for example `scratch/vX.Y.Z-pr-train-YYYYMMDD`.
- Use the scratch branch to merge or cherry-pick candidate PR heads in batches
  and learn which conflicts, tests, and overlaps are real.
- Treat the scratch branch as throwaway evidence — it collects noisy merge
  commits, partial conflict resolutions, and unrelated PR interactions, so ship
  from the release branch instead.
- After the scratch experiment, move only the safe result back to the release
  branch as narrow commits or direct merges. Keep each final commit explainable
  and testable.
- A PR that is clean against `main` is not necessarily clean against a release
  branch. Test mergeability against the branch that will actually receive the
  work.
- For already approved PRs, treat approval as a strong priority signal. Still
  inspect diffs, comments, check results, and release-branch conflicts before
  landing.

## Current Release Work

- Confirm the active branch for the current release lane from the latest handoff
  and `git branch --show-current`; recent work has landed on `main` through small
  PRs rather than a long-lived `codex/...` integration branch. This repo lives on
  multiple devices, so work in whichever local checkout you have and confirm the
  branch before editing.
- Read the workspace version from `Cargo.toml`; it advances per release lane.
- Base release triage on the current GitHub release milestone named in the active
  handoff (`gh issue list --repo Hmbown/CodeWhale --milestone "<current>" --state open`)
  unless Hunter gives a newer branch/milestone.
- Work the queue in this order: release blockers, recently approved PRs, clean
  PRs with small scope, blocked PRs with obvious fixes, dirty PRs that can be
  harvested safely, then larger architecture issues.
- Prefer batching PR conflict discovery on scratch branches, then harvesting
  reviewed, credited, tested slices back into the release branch.
- Before claiming an issue is done, verify whether the branch already contains
  equivalent work. If it does, prepare the GitHub note/closure path instead of
  reimplementing it.
- See `AGENTS.md` → "Where to work right now" for build/test commands, known
  suite papercuts, and the removed-machinery guardrails (agent-only surface,
  no lifecycle/coherence systems).

## 合并 `upstream/main` 到 `main-release` 注意事项

每次从上游合并时，必须检查以下定制化功能是否被上游覆盖/删除：

### CI / Workflow（必须保留 main-release 版本）

| 文件 | 说明 |
|------|------|
| `.github/workflows/release.yml` | fork 专用（不含 docker/Homebrew 自动发布），parity 检查针对 `main-release` |
| `.github/scripts/update-homebrew-tap.sh` | 指向 `tanuki-cat/homebrew-codewhale`，不是上游的 `Hmbown/homebrew-deepseek-tui` |
| `.github/workflows/homebrew.yml` | main-release 独有的 Homebrew 自动更新 workflow |

### Rust 源码定制（需重新集成）

| 模块 | 说明 |
|------|------|
| `crates/tui/src/seam_manager.rs` | 追加式分层上下文管理。上游删除后需保留文件并加 `mod` 声明，待后续集成到 engine |
| `crates/tui/src/tui/phase_strip.rs` | 上游 Ombre 路径只渲染 cost + cache。main-release 补齐了 balance、tokens、cost fallback |
| `crates/tui/src/tui/ui.rs` | `should_fetch_deepseek_balance` 不检查 `StatusItem::Balance`，DeepSeek 始终查余额；`fetch_deepseek_balance` 使用 `balance_origin()` 剥离路径后缀 |
| `crates/tui/src/pricing.rs` | 新增 `balance_origin()`；`provider_owned_hand_pricing_at` 中 DeepSeek 白名单改为 `starts_with("deepseek")` |
| `crates/tui/src/config.rs` | 新增 `drop_trailing_context_hint()` 剥离 harness 追加的 `[1m]`/`[128k]` context hint |
| `crates/tui/src/commands/groups/debug/balance.rs` | 上游替换为空壳 scaffold，需恢复 `balance_cell` 读取逻辑 |
| `crates/tui/src/localization.rs` + `locales/*.json` | 新增 `FooterCostPrefix` 消息 ID |
| `crates/tui/locales/` | 全部 8 个 locale 文件新增 `FooterCostPrefix` |

### 合并后验证

```bash
# 编译检查
RUSTFLAGS="-Dwarnings" cargo check --workspace --all-targets

# 关键功能验证
# 1. footer 中 cost / balance / tokens / cache 正常显示（Ombre 模式下）
# 2. /cost 命令返回正确金额
# 3. /balance 命令返回余额（非 scaffold 消息）
# 4. DeepSeek provider 所有模型（v4-pro、v4-flash、r1 等）均有定价
# 5. 模型名含 [1m] hint 时定价查找不受影响
```

### 上游已知删除了的 main-release 功能

- `AppAction::FetchBalance` / `balance_endpoint()` / `ProviderBalance` — 上游用事件循环内自动获取替代
- `plan_prompt.rs` — 上游 ModalKind/ViewEvent API 已重构，需重新适配

### 版本管理

- 合并后 bumper 次版本号（如 0.9.1 → 0.9.2）
- 打 annotated tag：`git tag -a vX.Y.Z -m "vX.Y.Z: merge upstream/main + main-release customizations"`
- Push：`git push origin main-release --tags`
