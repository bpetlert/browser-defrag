@_default:
  just --list

run +ARGS='':
  cargo run -- {{ARGS}}

# Run with debug log
run-debug +ARGS='':
  RUST_BACKTRACE=1 RUST_LOG=browser_defrag=debug cargo run -- {{ARGS}}

dry-run-firefox:
  RUST_BACKTRACE=1 RUST_LOG=browser_defrag=debug cargo run -- firefox --dry-run

dry-run-chromium:
  RUST_BACKTRACE=1 RUST_LOG=browser_defrag=debug cargo run -- chromium --dry-run

dry-run-unknown:
  RUST_BACKTRACE=1 RUST_LOG=browser_defrag=debug cargo run -- unknown --dry-run --profile-path=$HOME/.config/chromium

test +CASES='':
  RUST_BACKTRACE=1 RUST_LOG=browser_defrag=debug cargo test -- {{CASES}}

# Increase semver
bump-version VERSION:
  just _bump-cargo {{VERSION}}
  just _bump-pkgbuild {{VERSION}}
  cargo check

@_bump-cargo VERSION:
  cargo bump {{VERSION}}

@_bump-pkgbuild VERSION:
  sed -i -e "s/pkgver=.*/pkgver={{VERSION}}/g" -e "s/pkgrel=.*/pkgrel=1/g"  PKGBUILD.local
  sed -i -e "s/pkgver=.*/pkgver={{VERSION}}/g" -e "s/pkgrel=.*/pkgrel=1/g"  PKGBUILD.aur

# Commit bump version and release
release VERSION:
  git add Cargo.lock Cargo.toml PKGBUILD.aur PKGBUILD.local
  git commit --message="chore(release): {{VERSION}}"
  git tag --sign --annotate {{VERSION}} --message="version {{VERSION}}" --edit

# Update and audit dependencies
update-deps:
  cargo update
  cargo upgrade
  cargo audit

# Crate Arch package from GIT source
makepkg:
  makepkg -p PKGBUILD.local
  git co PKGBUILD.local

# Install in ~/.cargo/bin
install:
  cargo install --path .
  strip --strip-all ~/.cargo/bin/browser-defrag
