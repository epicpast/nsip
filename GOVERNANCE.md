# Governance

## Decision-Making Process

This project uses a maintainer-driven governance model.

### Roles

- **Maintainers**: Have full commit access and final say on project direction.
  Responsible for reviewing PRs, triaging issues, and cutting releases.
- **Contributors**: Submit PRs, report issues, and participate in discussions.
  Contributions are reviewed by maintainers before merging.

### How Decisions Are Made

1. **Minor changes** (bug fixes, documentation, small features): A single
   maintainer approval is sufficient.
2. **Significant changes** (new public API, architectural shifts, dependency
   additions): Require discussion in a GitHub issue or Discussion before
   implementation. At least one maintainer must approve.
3. **Breaking changes**: Require an RFC-style proposal in GitHub Discussions
   with a minimum review period of 7 days.

### RFC Process

For significant changes:

1. Open a GitHub Discussion with the `RFC` category
2. Describe the problem, proposed solution, and alternatives considered
3. Allow at least 7 days for community feedback
4. A maintainer will summarize the decision and close the discussion
5. Implementation proceeds via normal PR workflow

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development workflow details.
