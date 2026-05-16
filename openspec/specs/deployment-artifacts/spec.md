# deployment-artifacts Specification

## Purpose
TBD - created by archiving change nebula-ci-cd. Update Purpose after archive.
## Requirements
### Requirement: Release workflow runs on version tags

The repository SHALL provide a GitHub Actions release workflow triggered by Git tags beginning with `v`.

#### Scenario: Version tag is pushed

- **WHEN** a tag matching `v*` is pushed
- **THEN** the release workflow builds release artifacts

### Requirement: Release publishes FaaS OCI artifacts

The release workflow SHALL build each FaaS component and publish it to an OCI registry through `wkg push`.

#### Scenario: FaaS publication runs

- **WHEN** the release workflow reaches FaaS publication
- **THEN** it iterates over the crates in `faas/`
- **AND** runs `cargo component build --release`
- **AND** pushes each generated component to GHCR using the release tag

### Requirement: Release publishes VS Code extension artifact

The release workflow SHALL package the VS Code extension and attach the `.vsix` to the GitHub Release.

#### Scenario: VSIX is built

- **WHEN** the release workflow packages the extension
- **THEN** it runs the VS Code packaging tool
- **AND** uploads the generated `.vsix` using `softprops/action-gh-release`

### Requirement: Release can publish the CLI package

The release workflow SHALL publish the CLI npm package when an npm token is configured.

#### Scenario: NPM token is available

- **WHEN** `NPM_TOKEN` is configured
- **THEN** the release workflow runs `npm publish --workspace @nebula/cli --access public`

#### Scenario: NPM token is missing

- **WHEN** `NPM_TOKEN` is not configured
- **THEN** the release workflow skips npm publishing without failing the release

