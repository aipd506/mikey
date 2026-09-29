# Security Policy

## Reporting a Vulnerability

**Please do NOT open a public issue for security vulnerabilities.**

If you discover a security vulnerability in Mikey, please report it responsibly:

1. **Email:** Send a detailed report to **diveshpatil9104@gmail.com**
2. **Subject line:** `[SECURITY] Brief description of the vulnerability`
3. **Include:**
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)

## Response Timeline

- **Acknowledgment:** Within 48 hours of your report
- **Assessment:** Within 7 days, we'll provide an initial assessment
- **Fix:** We aim to release a fix within 30 days for confirmed vulnerabilities
- **Disclosure:** We'll coordinate disclosure timing with you

## Scope

The following are in scope for security reports:

| Component | Examples |
|-----------|---------|
| Wire protocol | Frame injection, buffer overflows, malformed packet handling |
| Session/trust | Token leakage, authentication bypass, unauthorized device access |
| PC binary | Privilege escalation, arbitrary code execution, DLL injection |
| Android app | Permission bypass, data leakage, intent hijacking |
| Network | Discovery beacon spoofing, man-in-the-middle on local network |

### Out of Scope

- Denial of service on the local network (Mikey is local-only by design)
- Social engineering attacks
- Vulnerabilities in dependencies - report these to the upstream project, but let us know so we can update
- Physical access attacks (if someone has physical access to your PC, Mikey's security is the least of your concerns)

## Security Design

Mikey's security model is documented in [`docs/SESSIONS_AND_TRUST.md`](../docs/SESSIONS_AND_TRUST.md). Key properties:

- **Local-only:** All traffic stays on the direct link between your phone and PC. No cloud relay, no internet-facing server.
- **Trust-on-first-use:** New devices require explicit approval on the PC. Trust is stored as a random 32-byte pairing token.
- **No persistent identifiers:** No tracking, no analytics, no telemetry.
- **Mic/camera default off:** Capture never starts without explicit user action.

## Supported Versions

| Version | Status |
|---------|--------|
| Latest release | Supported |
| Previous release | Supported (security fixes only) |
| Older versions | Unsupported |

## Recognition

We appreciate responsible disclosure. Security reporters will be credited in the release notes (unless they prefer to remain anonymous).

---

*Thank you for helping keep Mikey and its users safe.*
