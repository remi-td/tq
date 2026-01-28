# Licensing and Attribution Specifications

## Overview

This document specifies licensing requirements and third-party attribution for the `tq` (Teradata Query) tool. These requirements ensure legal compliance, transparency to users, and proper credit to upstream dependencies.

## Primary License

**REQ-LIC-001: MIT License**

The `tq` tool source code SHALL be licensed under the MIT License:

1. **REQ-LIC-001.1** - LICENSE file SHALL include full MIT License text
2. **REQ-LIC-001.2** - Copyright holder: The tq project contributors
3. **REQ-LIC-001.3** - License SHALL apply to all original source code in this repository
4. **REQ-LIC-001.4** - License SHALL NOT apply to third-party dependencies (covered separately below)

**MIT License Text Location:**
- Primary: `LICENSE` file in repository root
- Reference: README.md linking to LICENSE file

## Third-Party Dependencies and Attribution

**REQ-LIC-002: Dependency Attribution Requirements**

The tool depends on third-party software with separate licensing terms. Users SHALL be clearly informed of these dependencies and their licenses:

1. **REQ-LIC-002.1** - LICENSE file SHALL include section titled "Third-Party Dependencies"
2. **REQ-LIC-002.2** - Each dependency SHALL be listed with name, purpose, and license type
3. **REQ-LIC-002.3** - Dependencies with specific attribution requirements SHALL have full license text included
4. **REQ-LIC-002.4** - Users SHALL understand that using `tq` requires accepting dependency licenses

### Teradata Dependencies

**REQ-LIC-003: teradatarustapi Attribution**

The tool uses the `teradatarustapi` crate which bundles Teradata proprietary drivers:

1. **REQ-LIC-003.1** - Dependency name: teradatarustapi
2. **REQ-LIC-003.2** - Purpose: Rust API for Teradata database connections
3. **REQ-LIC-003.3** - License source: https://github.com/Teradata/teradatarustapi/blob/main/LICENSE
4. **REQ-LIC-003.4** - Attribution SHALL include full Teradata license text from upstream
5. **REQ-LIC-003.5** - Attribution SHALL note that Teradata GoSQL Driver is bundled within teradatarustapi

**REQ-LIC-004: Teradata GoSQL Driver License**

The teradatarustapi crate includes the Teradata GoSQL Driver:

1. **REQ-LIC-004.1** - Component name: Teradata GoSQL Driver
2. **REQ-LIC-004.2** - Bundled within: teradatarustapi crate
3. **REQ-LIC-004.3** - License type: Teradata proprietary license
4. **REQ-LIC-004.4** - License text source: https://github.com/Teradata/teradatarustapi/blob/main/LICENSE
5. **REQ-LIC-004.5** - Users SHALL be informed this is NOT an MIT-licensed component

**REQ-LIC-005: Go Runtime License**

The Teradata GoSQL Driver includes Go runtime components:

1. **REQ-LIC-005.1** - Component name: Go programming language runtime
2. **REQ-LIC-005.2** - License type: BSD-style Go license
3. **REQ-LIC-005.3** - License text source: https://github.com/Teradata/teradatarustapi/blob/main/THIRDPARTYLICENSE
4. **REQ-LIC-005.4** - Full Go license text SHALL be included in attribution
5. **REQ-LIC-005.5** - Attribution SHALL credit "The Go Authors"

## License File Structure

**REQ-LIC-006: LICENSE File Organization**

The LICENSE file SHALL be organized with clear sections:

1. **REQ-LIC-006.1** - Section 1: tq Tool License (MIT)
   - Full MIT license text
   - Copyright notice for tq project
   - Scope: "This license applies to the tq tool source code"

2. **REQ-LIC-006.2** - Section 2: Third-Party Dependencies
   - Introduction paragraph explaining dependency licenses
   - Subsection for each major dependency
   - Clear separation between tq license and dependency licenses

3. **REQ-LIC-006.3** - Section 3: teradatarustapi License
   - Full Teradata license text
   - Attribution to Teradata Corporation
   - Scope: "This license applies to the teradatarustapi crate and Teradata GoSQL Driver"

4. **REQ-LIC-006.4** - Section 4: Go License
   - Full Go license text (BSD-style)
   - Attribution to The Go Authors
   - Scope: "This license applies to Go runtime components bundled in Teradata GoSQL Driver"

5. **REQ-LIC-006.5** - Section 5: Other Rust Dependencies (Optional)
   - Notice that other Rust crates have their own licenses
   - Reference to cargo license tools for complete license list
   - Statement: "Most dependencies are MIT or Apache-2.0 licensed"

**Example Structure:**

```
# License

## tq Tool License

MIT License

Copyright (c) 2026 tq project contributors

[Full MIT license text...]

---

## Third-Party Dependencies

This software depends on third-party libraries with separate licensing terms.
By using tq, you accept the following dependency licenses:

### teradatarustapi and Teradata GoSQL Driver

The tq tool uses the teradatarustapi crate, which bundles the Teradata GoSQL
Driver. This component is licensed under Teradata's proprietary license:

[Full Teradata license text...]

### Go Programming Language

The Teradata GoSQL Driver includes components from the Go programming language,
licensed under the following BSD-style license:

[Full Go license text...]

---

## Additional Rust Dependencies

Other Rust crates used by tq have their own licenses (typically MIT or Apache-2.0).
For a complete list of dependencies and their licenses, run:

    cargo license --json

```

## User-Facing Messaging

**REQ-LIC-007: README License Section**

The README.md file SHALL include a clear license section:

1. **REQ-LIC-007.1** - Section title: "License"
2. **REQ-LIC-007.2** - Brief summary: "tq is MIT licensed, but depends on Teradata proprietary drivers"
3. **REQ-LIC-007.3** - Link to LICENSE file for full details
4. **REQ-LIC-007.4** - Statement: "By using this tool, you accept the license terms of all dependencies"
5. **REQ-LIC-007.5** - Highlight that Teradata components have separate license terms

**Example README Section:**

```markdown
## License

The `tq` tool source code is licensed under the MIT License. However, this tool
depends on the **teradatarustapi** crate, which includes Teradata's proprietary
GoSQL Driver and Go runtime components with separate license terms.

**Important:** By installing and using `tq`, you accept the license terms for:
- tq tool (MIT)
- Teradata GoSQL Driver (Teradata proprietary license)
- Go runtime (BSD-style Go license)

See the [LICENSE](LICENSE) file for complete license text and attributions.
```

## Installation Warnings

**REQ-LIC-008: Installation-Time Notice**

The tool SHALL NOT display license warnings during normal usage, but documentation SHALL make licensing clear:

1. **REQ-LIC-008.1** - Installation instructions SHALL mention license acceptance
2. **REQ-LIC-008.2** - No runtime license popups or warnings (silent acceptance by use)
3. **REQ-LIC-008.3** - cargo install command remains simple (no extra flags required)
4. **REQ-LIC-008.4** - Documentation SHALL be primary method of license communication

**Example Installation Section:**

```markdown
## Installation

Before installing, please review the [LICENSE](LICENSE) file. By installing tq,
you accept the terms for all bundled dependencies.

    cargo install tq
```

## Compliance Validation

**REQ-LIC-009: License Compliance Checks**

The project SHALL maintain license compliance through:

1. **REQ-LIC-009.1** - Regular review of LICENSE file accuracy (at least once per quarter)
2. **REQ-LIC-009.2** - Update LICENSE when dependencies change
3. **REQ-LIC-009.3** - Verify upstream license text matches included attribution
4. **REQ-LIC-009.4** - Check for new licensing requirements when updating teradatarustapi version
5. **REQ-LIC-009.5** - Maintain changelog of license-related changes

**REQ-LIC-010: Automated License Tooling**

The project MAY use automated tools to assist with license compliance:

1. **REQ-LIC-010.1** - `cargo license` command to list all dependency licenses
2. **REQ-LIC-010.2** - CI check to detect license changes in dependencies (optional)
3. **REQ-LIC-010.3** - Alerts when new dependencies with incompatible licenses are added (optional)

## Legal Disclaimer

**REQ-LIC-011: Warranty Disclaimer**

The LICENSE file SHALL include standard warranty disclaimers:

1. **REQ-LIC-011.1** - MIT license includes "AS IS" warranty disclaimer
2. **REQ-LIC-011.2** - No additional warranties beyond upstream dependency warranties
3. **REQ-LIC-011.3** - Liability limitations follow standard MIT license terms
4. **REQ-LIC-011.4** - No warranty for fitness for particular purpose

**REQ-LIC-012: Teradata Trademark Notice**

The LICENSE or README SHALL include trademark notice:

1. **REQ-LIC-012.1** - Statement: "Teradata is a trademark of Teradata Corporation"
2. **REQ-LIC-012.2** - Clarify that tq is NOT an official Teradata product
3. **REQ-LIC-012.3** - Clarify that tq is NOT endorsed by Teradata Corporation
4. **REQ-LIC-012.4** - Use of Teradata name is descriptive (compatibility/integration)

**Example Trademark Notice:**

```markdown
## Trademarks

Teradata is a registered trademark of Teradata Corporation. This project is not
affiliated with, endorsed by, or sponsored by Teradata Corporation. The name
"Teradata" is used solely to indicate compatibility with Teradata database systems.
```

## Future Considerations

**REQ-LIC-013: License Evolution**

As the project evolves, licensing SHALL be reviewed when:

1. **REQ-LIC-013.1** - Adding new direct dependencies with different licenses
2. **REQ-LIC-013.2** - Updating teradatarustapi to new major version
3. **REQ-LIC-013.3** - Contributing to the Teradata Rust ecosystem
4. **REQ-LIC-013.4** - Distributing pre-compiled binaries (additional considerations)
5. **REQ-LIC-013.5** - Creating Docker images or other distribution formats

**REQ-LIC-014: Community Contributions**

For external contributors:

1. **REQ-LIC-014.1** - Contributors implicitly license contributions under MIT
2. **REQ-LIC-014.2** - CONTRIBUTING.md SHALL clarify license grant
3. **REQ-LIC-014.3** - No CLA (Contributor License Agreement) required for simple contributions
4. **REQ-LIC-014.4** - Large contributions MAY require explicit license grant acknowledgment

## Acceptance Criteria

The licensing implementation is complete when:

1. LICENSE file includes all required sections and license texts
2. README.md includes clear license summary and links
3. Trademark notice is present and accurate
4. Third-party attribution includes Teradata and Go licenses
5. License text matches upstream sources
6. Users can clearly understand licensing obligations before installation
7. No misleading claims about licensing (e.g., "pure MIT" when dependencies have other licenses)

---

**Last Updated:** 2026-01-27
