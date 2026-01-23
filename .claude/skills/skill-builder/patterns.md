# Common Skill Patterns

Proven patterns for effective skills.

## Pattern 1: Domain Expertise Skill

For encoding specialized knowledge:

```yaml
---
name: legal-document-review
description: Reviews legal documents for common issues. Use when user asks to review contracts, agreements, or legal documents.
---

# Legal Document Review

## Review Process

1. Identify document type and jurisdiction
2. Check for standard clauses
3. Flag missing or non-standard provisions
4. Review for clarity and enforceability

[Specific procedures follow...]
```

## Pattern 2: Workflow Skill

For multi-step processes with checklist:

```yaml
---
name: api-integration-builder
description: Guides building API integrations. Use when creating new API clients or integrations.
---

# API Integration Builder

## Workflow
Copy this checklist:
- [ ] Step 1: Design client architecture
- [ ] Step 2: Implement authentication
- [ ] Step 3: Add error handling
- [ ] Step 4: Write tests
- [ ] Step 5: Document usage

**Step 1: Design client architecture**
[Details...]

**Step 2: Implement authentication**
[Details...]
```

## Pattern 3: Tool Skill with Scripts

For specific tool usage with utility scripts:

```yaml
---
name: presentation-creator
description: Creates PowerPoint presentations. Use when user asks to create slides or PPTX files.
---

# Presentation Creator

## Creation Process

1. Understand requirements
2. Structure content: `python scripts/outline.py`
3. Apply design: `python scripts/style.py`
4. Generate PPTX: `python scripts/generate.py`

## Utility Scripts

**outline.py**: Analyze content and create slide structure
**style.py**: Apply consistent design theme
**generate.py**: Create final PPTX file
```

## Pattern 4: Progressive Disclosure

For skills with extensive reference material:

```yaml
---
name: bigquery-analyst
description: Analyzes data in BigQuery. Use when user asks about database queries or data analysis.
---

# BigQuery Analysis

## Quick Start
Run queries with: `bq query --use_legacy_sql=false 'SELECT ...'`

## Dataset References
**Finance**: See [reference/finance.md](reference/finance.md)
**Sales**: See [reference/sales.md](reference/sales.md)
**Product**: See [reference/product.md](reference/product.md)
```

Claude loads the specific reference only when needed.

## Pattern 5: Template + Examples

For consistent output formatting:

```yaml
---
name: commit-message-writer
description: Writes conventional commit messages. Use when committing code changes.
---

# Commit Message Writer

## Template
```
type(scope): brief description

Detailed explanation if needed
```

## Examples

**Input:** Added user authentication with JWT tokens
**Output:**
```
feat(auth): implement JWT-based authentication

Add login endpoint and token validation middleware
```

**Input:** Fixed bug where dates displayed incorrectly
**Output:**
```
fix(reports): correct date formatting in timezone conversion

Use UTC timestamps consistently across report generation
```
```

## Pattern 6: Conditional Workflow

For different paths based on context:

```markdown
## Processing Workflow

**If input is PDF:**
1. Extract text: `python scripts/extract_pdf.py`
2. Parse structure
3. Generate output

**If input is image:**
1. OCR text: `python scripts/ocr_image.py`
2. Validate extraction
3. Generate output

**If input is text:**
1. Skip extraction
2. Parse directly
3. Generate output
```
