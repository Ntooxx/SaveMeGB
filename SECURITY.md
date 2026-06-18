# Security Policy

## Supported versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | ✅ Active          |

## Reporting a vulnerability

SaveMeGB is a file deletion tool. A bug in the engine could cause real data loss. We take security seriously.

**Please do not open a public issue for security problems.** Instead:

- Email the maintainers (see GitHub profile)
- Or open a [GitHub Security Advisory](https://github.com/Ntooxx/SaveMeGB/security/advisories/new) (private, only maintainers see it)

Please include:
- What you found
- How to reproduce
- Potential impact
- Suggested fix (if you have one)

We aim to respond within 48 hours.

## What we consider a security issue

- **Any way the app could delete files the user did not explicitly authorize**
- **Any way the app could send user data off-device without consent** (we currently send a manifest update request to ludusavi's GitHub, that's it)
- **Any way the app could be tricked into running arbitrary code** (e.g., via malicious scan input)
- **License validation bypasses** (for the Pro tier)

## Out of scope

- Bugs in features that don't affect data integrity
- UI / cosmetic issues
- Performance problems
- Feature requests

## What we'll do

1. Acknowledge your report within 48 hours
2. Investigate and develop a fix
3. Credit you in the fix release notes (unless you prefer to stay anonymous)
4. Publish a CVE if the issue is severe enough
