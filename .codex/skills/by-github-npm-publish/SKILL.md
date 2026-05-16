---
name: by-github-npm-publish
description: Set up or operate a GitHub Actions release workflow that builds multi-platform artifacts, creates or updates a GitHub Release, and publishes a package to npm. Use when Codex needs to design, document, debug, or run a GitHub-based build and publish pipeline for CLI or package distribution.
---

# By GitHub Npm Publish

Build the release flow around four jobs: metadata validation, matrix builds, GitHub Release publishing, and npm publishing. Keep the build job responsible only for compiling and uploading artifacts; do the final packaging and publishing in dedicated downstream jobs.

## Workflow Shape

Implement the workflow in this order:

1. `prepare`
2. `build`
3. `release`
4. `publish-npm`

Use `workflow_dispatch` inputs for manual releases and optionally add tag-push triggers such as `v*.*.*` for automatic releases.

Recommended manual inputs:

- `ref`: git ref to build
- `release_tag`: version tag like `v1.2.3`
- `publish_release`: whether to create or update GitHub Release
- `publish_npm`: whether to publish to npm

## Prepare Job

Use the `prepare` job to validate release intent before any expensive build work starts.

Validate:

- `release_tag` exists when any publish flag is true
- `release_tag` matches the expected tag pattern
- tag version matches the package version
- target commit is on the default branch history for official releases
- required secrets exist before enabling publish jobs

Emit normalized outputs for downstream jobs:

- checkout ref
- target sha
- release tag
- package version
- publish booleans

Fail here rather than inside publish jobs if any release precondition is wrong.

## Build Job

Use a matrix build for each target platform. Each matrix leg should:

- check out the target ref
- install the target toolchain
- build the binary or package artifact
- package the platform-specific distributable
- upload a release artifact
- upload a second artifact for npm packaging if npm publishing needs the compiled outputs

Keep release assets and npm staging artifacts separate. Use different artifact names and directories so later jobs can download them independently.

For example:

- release asset artifacts: final `.tar.gz` or `.zip`
- npm binary artifacts: unpacked platform binaries arranged in a predictable directory layout

## Release Job

Run this job only when `publish_release` is true.

Sequence:

1. Checkout the same ref used for the build.
2. Download release asset artifacts.
3. Download npm packaging artifacts if the release should also include a package tarball.
4. Build the npm tarball with `npm pack` if needed.
5. Copy all final release files into a single bundle directory.
6. Create or update the GitHub Release.

Critical ordering rules:

- Do `actions/checkout` before `actions/download-artifact`, because checkout can clean the workspace.
- If `npm pack` runs a `prepack` script, copy already-downloaded release assets into a final bundle directory before packing, so packaging side effects do not remove them.
- Upload from one final bundle directory rather than mixing sources at release time.

When implementing create-or-update behavior, prefer:

- `gh release view` to detect the tag
- `gh release upload --clobber` to replace assets on an existing release
- `gh release create` when the release does not yet exist

## npm Publish Job

Run this job only when `publish_npm` is true.

Sequence:

1. Checkout the target ref.
2. Set up Node with the npm registry URL.
3. Download npm binary artifacts.
4. Publish the package with `npm publish`.

Pass credentials through:

- `NODE_AUTH_TOKEN`

Store the token as a GitHub Actions secret, commonly `NPM_TOKEN`.

Use an npm automation token when the npm account enforces 2FA.

## Package Version Rules

Keep versioning strict:

- package version must already be updated in the repo before release
- release tag version must match the package version exactly
- do not publish a version that already exists on npm unless the workflow is intentionally update-only for GitHub Release assets

For npm packages, the usual tag format is:

```text
v<package-version>
```

## Recommended Artifact Strategy

Use this split:

- build job produces platform release archives
- build job also produces raw compiled outputs for package assembly
- release job aggregates final files into one upload directory
- npm publish job reconstructs the package from downloaded build artifacts

This avoids rebuilding in publish jobs and keeps release outputs reproducible from the same compiled inputs.

## Manual Operating Modes

Support these three modes:

1. Build only
2. Publish GitHub Release only
3. Publish GitHub Release and npm together

Use them like this:

- Build only: provide `ref`, leave publish flags false
- Release only: provide `ref`, `release_tag`, and enable `publish_release`
- Release plus npm: provide `ref`, `release_tag`, and enable both publish flags

## Common Failure Modes

Check these first when the workflow fails:

- version mismatch between `release_tag` and package version
- publish job enabled without `NPM_TOKEN`
- target commit not on default branch history
- checkout happening after artifact download
- `npm pack` or `prepack` removing downloaded files
- npm version already published
- artifact naming or directory layout mismatch between build and publish jobs

When debugging, inspect the failing job's downloaded artifact paths and verify that each downstream job expects the same structure that upstream jobs uploaded.

## Implementation Guidelines

Prefer these patterns:

- use `actions/upload-artifact` and `actions/download-artifact` for job boundaries
- use explicit `if:` conditions for publish jobs
- use `prepare` outputs rather than recomputing release state in multiple jobs
- use one final `release-bundle` directory before uploading Release assets
- parse `npm pack --json` defensively if package scripts may print extra logs

Avoid these patterns:

- combining checkout and artifact download in the wrong order
- rebuilding the project separately in release and npm publish jobs
- mixing package assembly state with raw artifact download directories
- relying on implicit branch or tag assumptions inside later jobs

## Verification

After a successful run, verify:

- all expected platform assets exist on GitHub Release
- the npm package exists in the registry
- the npm `latest` tag points to the intended version
- the installed CLI or package works from the published artifact

Useful checks:

```bash
gh release view <tag>
npm view <package-name> version
npm view <package-name> dist-tags --json
```
