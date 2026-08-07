# Phase 6: Framework Optimization (Action-Only Mode)

**Owner:** Sprint Coordinator (Main Agent)
**Goal:** Keep the agentic framework fast, lean, and effective by directly fixing friction observed during the sprint.

## Action-Only Contract
Phase 6 MUST NOT generate prose proposal files (`sprint-N-optimizations.md`). It operates strictly via **direct edits**:

1. **Review Sprint Execution:**
   - Did any sub-agent make repeated mistakes, get confused, or re-read files multiple times?
   - Did metrics show unusual token spikes or broken tools?

2. **Execute Direct Fix (If Friction Detected):**
   - Directly edit the relevant `.agents/skills/*` file, agent prompt file, script, or code in that turn.
   - Example: Fix a bug in a metric script, update a vague sub-agent instruction, or refine a CLI specification tip.

3. **Immediate Exit (If Operating Cleanly):**
   - If no workflow friction occurred, output:
     ```
     Phase 6 Assessment: No framework friction observed in Sprint N. Workflow operating at peak efficiency.
     ```
   - Exit Phase 6 immediately in 0 additional turns.

4. **Commit Changes (If edits made):**
   ```bash
   git add .
   git commit -m "Sprint N: Framework optimization - [short description of fix]"
   git push origin main
   ```
