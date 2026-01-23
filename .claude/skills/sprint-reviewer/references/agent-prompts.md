# Sprint Review Agent Prompts

## rust-teradata-architect (Technical Review)

```
Sprint [N] technical review.

Review scope:
1. Implementation approach and architectural decisions
2. Code quality, modularity, maintainability
3. Technical challenges and solutions
4. Technical debt assessment
5. Adherence to docs/design/*.md (design documentation)

Provide recommendations for:
- Code improvements
- Architectural refinements
- rust-coder skill enhancements
```

## quality-validator (Quality Review)

```
Sprint [N] quality review.

Review scope:
1. Test coverage analysis
2. Test pass rate and failures
3. Testing methodology effectiveness
4. Regression testing results

Provide recommendations for:
- Testing approach improvements
- testing-guidelines.md updates
- Automated testing infrastructure
```

## cli-ux-designer (UX Review)

```
Sprint [N] UX review.

Review scope:
1. Feature usability
2. CLI design consistency
3. Flag naming and options
4. Help text quality
5. Error messages

Provide recommendations for:
- UX improvements
- specifications.md updates
- Documentation updates
```

## How to Launch

Launch all 3 in a SINGLE message with multiple Task calls:
- Parallel execution saves time
- Each agent reviews their domain
- Coordinator consolidates findings
