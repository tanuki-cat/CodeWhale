#!/usr/bin/env bash
# Update the Homebrew tap at tanuki-cat/homebrew-codewhale after a release.
#
# Expected environment:
#   TAG       – git tag, e.g. "v0.8.31"
#   MANIFEST  – path to codewhale-artifacts-sha256.txt
#   TAP_REPO  – owner/repo of the Homebrew tap
#   TOKEN     – PAT with contents:write on TAP_REPO (optional; skips if unset)
#
# Re-releasing the same version (for example force-moving the v0.8.63 tag onto a
# new commit and rebuilding the binaries) keeps the Homebrew `version`
# unchanged. Homebrew decides upgrades from the `version` (plus `revision`), not
# from the asset checksums, so `brew upgrade` would treat a rebuilt release as
# "already installed". To make same-version rebuilds upgradable we bump the
# formula `revision` whenever the generated formula content changes but the
# version does not, and reset it when the version changes.

set -euo pipefail

: "${TAG:?}"
: "${MANIFEST:?}"
: "${TAP_REPO:?}"

if [ -z "${TOKEN:-}" ]; then
  echo "No Homebrew tap token configured; skipping."
  exit 0
fi

VERSION="${TAG#v}"

die() { echo "::error::${1}" >&2; exit 1; }

sha() {
  local file="${1:?}"
  local val
  val="$(awk -v f="${file}" '$2 == f {print $1; exit}' "${MANIFEST}")"
  if [ -z "${val}" ]; then
    die "Missing binary in checksum manifest: ${file}"
  fi
  echo "${val}"
}

# --- read checksums ---------------------------------------------------

# Canonical dispatcher and TUI
readonly SHA_COD_MACOS_ARM="$(sha codewhale-macos-arm64)"
readonly SHA_TUI_MACOS_ARM="$(sha codewhale-tui-macos-arm64)"
readonly SHA_COD_MACOS_X64="$(sha codewhale-macos-x64)"
readonly SHA_TUI_MACOS_X64="$(sha codewhale-tui-macos-x64)"
readonly SHA_COD_LINUX_ARM="$(sha codewhale-linux-arm64)"
readonly SHA_TUI_LINUX_ARM="$(sha codewhale-tui-linux-arm64)"
readonly SHA_COD_LINUX_X64="$(sha codewhale-linux-x64)"
readonly SHA_TUI_LINUX_X64="$(sha codewhale-tui-linux-x64)"

readonly BASE_URL="https://github.com/tanuki-cat/CodeWhale/releases/download/${TAG}"

# Emit the formula. Argument $1 is the Homebrew revision; when it is 0 the
# `revision` line is omitted (Homebrew treats a missing revision as 0).
gen_formula() {
  local revision="${1:?}"
  {
    echo 'class Codewhale < Formula'
    echo '  desc "Terminal-native coding agent for any model — open models first"'
    echo '  homepage "https://github.com/tanuki-cat/CodeWhale"'
    echo "  version \"${VERSION}\""
    echo '  license "MIT"'
    if [ "${revision}" -gt 0 ]; then
      echo "  revision ${revision}"
    fi
    cat << EOF

  on_macos do
    if Hardware::CPU.arm?
      url "${BASE_URL}/codewhale-macos-arm64", using: :nounzip
      sha256 "${SHA_COD_MACOS_ARM}"
      resource "tui" do
        url "${BASE_URL}/codewhale-tui-macos-arm64", using: :nounzip
        sha256 "${SHA_TUI_MACOS_ARM}"
      end
    else
      url "${BASE_URL}/codewhale-macos-x64", using: :nounzip
      sha256 "${SHA_COD_MACOS_X64}"
      resource "tui" do
        url "${BASE_URL}/codewhale-tui-macos-x64", using: :nounzip
        sha256 "${SHA_TUI_MACOS_X64}"
      end
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "${BASE_URL}/codewhale-linux-arm64", using: :nounzip
      sha256 "${SHA_COD_LINUX_ARM}"
      resource "tui" do
        url "${BASE_URL}/codewhale-tui-linux-arm64", using: :nounzip
        sha256 "${SHA_TUI_LINUX_ARM}"
      end
    else
      url "${BASE_URL}/codewhale-linux-x64", using: :nounzip
      sha256 "${SHA_COD_LINUX_X64}"
      resource "tui" do
        url "${BASE_URL}/codewhale-tui-linux-x64", using: :nounzip
        sha256 "${SHA_TUI_LINUX_X64}"
      end
    end
  end

  def install
    bin.install Dir["*"].first => "codewhale"
    resource("tui").stage { bin.install Dir["*"].first => "codewhale-tui" }
  end

  test do
    system "#{bin}/codewhale", "--version"
  end
end
EOF
  }
}

# --- temp dirs --------------------------------------------------------

FORMULA_FILE="$(mktemp)"
TAP_DIR="$(mktemp -d)"
trap 'rm -rf "${TAP_DIR}" "${FORMULA_FILE}"' EXIT

# --- clone tap first so we can read the existing formula ---------------

ENCODED_TOKEN="$(printf '%s' "${TOKEN}" | python3 -c 'import sys,urllib.parse;print(urllib.parse.quote(sys.stdin.read(),safe=""))')"
TAP_URL="https://x-access-token:${ENCODED_TOKEN}@github.com/${TAP_REPO}.git"

git clone --depth 1 "${TAP_URL}" "${TAP_DIR}"

EXISTING="${TAP_DIR}/Formula/codewhale.rb"

# Drop any `revision N` line so formula *content* can be compared independent of
# the revision counter.
strip_revision() {
  sed -E '/^[[:space:]]*revision[[:space:]]+[0-9]+[[:space:]]*$/d' "${1}"
}

# --- decide the revision ----------------------------------------------

REVISION=0
if [ -f "${EXISTING}" ]; then
  OLD_VERSION="$(sed -nE 's/^[[:space:]]*version[[:space:]]+"([^"]+)".*/\1/p' "${EXISTING}" | head -1)"
  OLD_REVISION="$(sed -nE 's/^[[:space:]]*revision[[:space:]]+([0-9]+).*/\1/p' "${EXISTING}" | head -1)"
  OLD_REVISION="${OLD_REVISION:-0}"

  # Compare new content (no revision line) against the existing formula with its
  # revision line stripped. If identical, there is genuinely nothing to ship.
  if diff -q <(gen_formula 0) <(strip_revision "${EXISTING}") >/dev/null 2>&1; then
    echo "Formula unchanged (already at ${VERSION}, revision ${OLD_REVISION}); nothing to push."
    exit 0
  fi

  if [ "${OLD_VERSION}" = "${VERSION}" ]; then
    REVISION=$((OLD_REVISION + 1))
    echo "Same version ${VERSION} with changed assets; bumping revision ${OLD_REVISION} -> ${REVISION}."
  else
    REVISION=0
    echo "New version ${VERSION} (was ${OLD_VERSION:-none}); resetting revision to 0."
  fi
fi

gen_formula "${REVISION}" > "${FORMULA_FILE}"

# --- push to tap repo --------------------------------------------------

mkdir -p "${TAP_DIR}/Formula"
cp "${FORMULA_FILE}" "${TAP_DIR}/Formula/codewhale.rb"

cd "${TAP_DIR}"
git config user.name  "github-actions[bot]"
git config user.email "github-actions[bot]@users.noreply.github.com"

git add Formula/codewhale.rb

if git diff --cached --quiet; then
  echo "Formula unchanged; nothing to push."
  exit 0
fi

if [ "${REVISION}" -gt 0 ]; then
  COMMIT_MSG="chore: rebuild formula for ${VERSION} (revision ${REVISION})"
else
  COMMIT_MSG="chore: bump formula to ${VERSION}"
fi

git commit -m "${COMMIT_MSG}

Automated update from the release workflow."

git push origin HEAD:main
echo "Pushed formula update to ${TAP_REPO} (v${VERSION}, revision ${REVISION})"
