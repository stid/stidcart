# Security

stidcart reads cartridge images and program files that may come from anywhere, so a file that
makes it crash, hang or write outside the paths it was given is a security bug.

Please report one privately, through
[a security advisory](https://github.com/stid/stidcart/security/advisories/new), and not in a
public issue. Say which version you ran (the app's About box shows it, and so does the "built by"
line of `stidcart list` on a cart it made), on which system, and attach the file if you can.

This is a hobby project with one maintainer. I read every report and fix what I can reproduce, with
no promise on timing. Only the latest release gets fixes.

The macOS downloads are signed with a Developer ID and notarized by Apple. The Windows and Linux
downloads are not signed; `SHA256SUMS` on each release lists what the files should hash to.
