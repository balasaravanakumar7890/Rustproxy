# Security Policy & Responsible Use Guidelines

## Scope & Defensive Intent

**RustProxy** is designed, developed, and maintained exclusively for authorized defensive security testing, internal software development, network protocol debugging, and educational security research.

### Permitted Capabilities
- Intercepting, inspecting, and modifying traffic on applications you own or are explicitly authorized to test in writing.
- Local application debugging and security configuration diagnostics.
- Passive vulnerability analysis and missing security control detection.

### Explicit Prohibitions
- Bypassing access controls, authentication gates, or software licensing.
- Creating malware, exploits, or weaponized payload delivery systems.
- Conducting denial-of-service, destructive fuzzing, or unauthorized testing against third-party systems.

---

## Supported Versions

| Version | Supported | Security Updates |
| :--- | :--- | :--- |
| **1.0.x** | :white_check_mark: | Supported (Active) |
| < 1.0 | :x: | Unsupported |

---

## Reporting a Vulnerability

If you discover a security vulnerability in RustProxy (such as an unauthorized local API privilege escalation, sandbox bypass, or unsafe memory handling):

1. **Do not create a public GitHub issue.**
2. Report the vulnerability privately to the project maintainer:
   - **Contact**: `balasaravanakumar7890@users.noreply.github.com`
   - Include a detailed technical description, proof of concept, and affected environment.
3. The maintainer will acknowledge receipt within 48 hours and coordinate a coordinated disclosure timeline and security patch.
