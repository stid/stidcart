# Releasing

How a stidcart release is cut, how the Homebrew tap follows it, and how the macOS builds are
signed. This is for whoever maintains the repository; building and testing are in
[CONTRIBUTING.md](CONTRIBUTING.md).

## Cutting a release

A release is built by `.github/workflows/release.yml` from a tag on `main`.

1. In a pull request, set the new version in `Cargo.toml` (`[workspace.package]`) and in
   `gui/src-tauri/Cargo.toml`, and let both `Cargo.lock` files follow. The two must agree:
   `docs/lcd/ci/release-version.sh` prints the version and fails if they do not.
2. Once it is merged, tag that commit `v` plus the version and push the tag:
   `git tag v0.31.0 && git push origin v0.31.0`.
3. The workflow builds the command for macOS, Linux and Windows, the macOS app as a `.dmg`, the
   Linux app as an AppImage and a `.deb` and the Windows app as an installer, and attaches them
   with a `SHA256SUMS` file to a draft release. Read the draft, try a download, then publish it.
4. Publishing runs `.github/workflows/tap.yml`, which points the Homebrew tap at the new release
   (see [The Homebrew tap](#the-homebrew-tap)). Its run and the tap's own `Tests` run should both
   end green.

The workflow refuses a tag that names another version or a commit that is not on `main`. A pull
request that changes the workflow runs the same build as a dry run: nothing is released, and the
files are in the run's `release` artifact. To repeat the run for a tag, delete the draft first.

A `v*` tag cannot be moved or deleted once pushed, and a published release cannot be changed:
its files and its tag are locked (D-078). Homebrew checks each download against the checksum in
the tap, so a file is never replaced. A release that went out wrong is followed by a new version.
A tag pushed on the wrong commit is removed by switching off the "release tags" ruleset in the
repository's settings for as long as that takes.

## The Homebrew tap

[stid/homebrew-tap](https://github.com/stid/homebrew-tap) holds two recipes: formula `stidcart`
for the command and cask `stidcart-app` for the app. Each names a release's macOS file and its
checksum. When a release is published, `tap.yml` rewrites both from that release's `SHA256SUMS`
and pushes, and the tap's `Tests` workflow then installs both on a Mac that has never had
stidcart on it. A prerelease leaves the tap alone.

The push needs one repository secret, `TAP_TOKEN`: a fine-grained personal access token limited to
the `homebrew-tap` repository, with read and write access to its contents and nothing else. Make
it under GitHub's Settings, Developer settings, Personal access tokens, then set it from a
terminal:

```bash
gh secret set TAP_TOKEN   # asks for the value
```

The token expires on the date chosen when it was made. After that a release still publishes and
only the tap update fails. Set a new token, then repeat the update from the Actions tab: "Tap",
"Run workflow", with the release's tag. A pull request that changes `tap.yml` dry-runs it against
the latest release, printing the change without pushing.

## Signing the macOS builds

The macOS app and command are signed with a Developer ID and notarized, from five repository
secrets. A run without them (a fork's pull request) still builds, with the app signed ad hoc, and a
tag refuses to build that way. Setting them up takes an Apple Developer Program membership and is
done once:

1. Make a certificate request: Keychain Access, Certificate Assistant, "Request a Certificate From
   a Certificate Authority", saved to disk.
2. At developer.apple.com, under Certificates, add a "Developer ID Application" certificate from
   that request. Download it and open it, which puts it in the login keychain.
3. In Keychain Access, under My Certificates, export it together with its private key as a `.p12`
   with a password.
4. In App Store Connect, under Users and Access, Integrations, create a team API key. Download its
   `.p8` file, which is offered once, and note the key ID and the issuer ID.
5. Set the secrets from a terminal, so that none of them passes through anything else:

   ```bash
   base64 -i certificate.p12 | gh secret set APPLE_CERTIFICATE
   gh secret set APPLE_CERTIFICATE_PASSWORD   # asks for the value
   gh secret set APPLE_API_KEY                # the key ID
   gh secret set APPLE_API_ISSUER             # the issuer ID
   gh secret set APPLE_API_KEY_P8 <AuthKey_XXXXXXXXXX.p8
   ```

The certificate expires (the one in use on 2027-12-16), and the developer site shows the date.
After that nothing new can be signed until steps 1 to 3 are repeated and the two certificate
secrets set again; builds already released keep opening. Keep the `.p12` and the `.p8` somewhere
safe and out of the repository.

The Windows installer is not signed, so SmartScreen stops it with "Windows protected your PC":
"More info", then "Run anyway". It installs for the current user and asks for no administrator
rights.
