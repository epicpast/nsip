---
diataxis_type: reference
---
# GitHub Template Repository Features

> What copies when someone clicks **"Use this template"** — and what doesn't.

This reference covers every category of GitHub repository configuration, whether it transfers to new repositories created from templates, and workarounds for items that don't copy.

**Source:** [Creating a repository from a template](https://docs.github.com/en/repositories/creating-and-managing-repositories/creating-a-repository-from-a-template) · [Creating a template repository](https://docs.github.com/en/repositories/creating-and-managing-repositories/creating-a-template-repository)

---

## 1. Repository Files and Structure

| Item | Copies? | Configuration File | Docs |
|------|---------|-------------------|------|
| All files and folders | Yes | N/A | [Creating a repository from a template](https://docs.github.com/en/repositories/creating-and-managing-repositories/creating-a-repository-from-a-template) |
| Hidden files (.gitignore, .github/, etc.) | Yes | N/A | [Creating a template repository](https://docs.github.com/en/repositories/creating-and-managing-repositories/creating-a-template-repository) |
| Default branch | Yes | N/A | [Creating a template repository](https://docs.github.com/en/repositories/creating-and-managing-repositories/creating-a-template-repository) |
| All branches | Optional | User selects "Include all branches" during creation | [Creating a repository from a template](https://docs.github.com/en/repositories/creating-and-managing-repositories/creating-a-repository-from-a-template) |
| Git commit history | **No** | N/A — starts with single commit | [Creating a repository from a template](https://docs.github.com/en/repositories/creating-and-managing-repositories/creating-a-repository-from-a-template) |
| Git LFS files | **No** | N/A — template restriction | [Creating a template repository](https://docs.github.com/en/repositories/creating-and-managing-repositories/creating-a-template-repository) |

---

## 2. Branch Protection Rules

| Item | Copies? | Configuration File | Docs |
|------|---------|-------------------|------|
| Branch protection rules | **No** | N/A — UI/API only | [Managing a branch protection rule](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/managing-a-branch-protection-rule) |
| Repository rulesets | **No** | Can export/import as JSON | [Managing rulesets for a repository](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/managing-rulesets-for-a-repository) |
| Organization-level rulesets | N/A | Apply automatically to all org repos | [Managing rulesets for repositories in your organization](https://docs.github.com/en/organizations/managing-organization-settings/managing-rulesets-for-repositories-in-your-organization) |

> **Note:** There is no native `.github/settings.yml` in GitHub. That is a third-party tool ([Probot Settings App](https://probot.github.io/apps/settings/)).

---

## 3. Issues, PRs, and Templates

| Item | Copies? | Configuration File | Docs |
|------|---------|-------------------|------|
| Issue templates | Yes | `.github/ISSUE_TEMPLATE/*.md` or `.yml` | [Configuring issue templates](https://docs.github.com/en/communities/using-templates-to-encourage-useful-issues-and-pull-requests/configuring-issue-templates-for-your-repository) |
| Issue forms config | Yes | `.github/ISSUE_TEMPLATE/config.yml` | [Syntax for issue forms](https://docs.github.com/en/communities/using-templates-to-encourage-useful-issues-and-pull-requests/syntax-for-issue-forms) |
| PR templates | Yes | `.github/PULL_REQUEST_TEMPLATE.md` | [Creating a PR template](https://docs.github.com/en/communities/using-templates-to-encourage-useful-issues-and-pull-requests/creating-a-pull-request-template-for-your-repository) |
| Existing issues | **No** | N/A — data, not files | [Creating a repository from a template](https://docs.github.com/en/repositories/creating-and-managing-repositories/creating-a-repository-from-a-template) |
| Existing pull requests | **No** | N/A — data, not files | [Creating a repository from a template](https://docs.github.com/en/repositories/creating-and-managing-repositories/creating-a-repository-from-a-template) |

---

## 4. Security Settings

| Item | Copies? | Configuration File | Docs |
|------|---------|-------------------|------|
| SECURITY.md | Yes | `SECURITY.md`, `.github/SECURITY.md`, or `docs/SECURITY.md` | [Adding a security policy](https://docs.github.com/en/code-security/getting-started/adding-a-security-policy-to-your-repository) |
| Dependabot config | Yes | `.github/dependabot.yml` | [Dependabot configuration options](https://docs.github.com/en/code-security/dependabot/dependabot-version-updates/configuration-options-for-the-dependabot.yml-file) |
| Secret scanning config | Yes | `.github/secret_scanning.yml` | [Enabling secret scanning](https://docs.github.com/en/code-security/secret-scanning/enabling-secret-scanning-features/enabling-secret-scanning-for-your-repository) |
| Code scanning workflows | Yes | `.github/workflows/codeql-analysis.yml` | [Configuring code scanning](https://docs.github.com/en/code-security/code-scanning/creating-an-advanced-setup-for-code-scanning/configuring-advanced-setup-for-code-scanning) |
| Security feature toggles | **No** | UI/API settings only | [Managing security settings](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/enabling-features-for-your-repository/managing-security-and-analysis-settings-for-your-repository) |
| Secret scanning enabled | **No** | UI/API settings only | [Enabling secret scanning](https://docs.github.com/en/code-security/secret-scanning/enabling-secret-scanning-features/enabling-secret-scanning-for-your-repository) |
| Dependabot alerts enabled | **No** | UI/API settings only | [About Dependabot security updates](https://docs.github.com/en/code-security/dependabot/dependabot-security-updates/about-dependabot-security-updates) |

> Security *configuration files* copy. Security *feature toggles* (enabled/disabled) do **not**.

---

## 5. Community Health Files

| Item | Copies? | Configuration File | Docs |
|------|---------|-------------------|------|
| CODE_OF_CONDUCT.md | Yes | Root, `.github/`, or `docs/` | [Adding a code of conduct](https://docs.github.com/en/communities/setting-up-your-project-for-healthy-contributions/adding-a-code-of-conduct-to-your-project) |
| CONTRIBUTING.md | Yes | Root, `.github/`, or `docs/` | [Default community health files](https://docs.github.com/en/communities/setting-up-your-project-for-healthy-contributions/creating-a-default-community-health-file) |
| SUPPORT.md | Yes | Root, `.github/`, or `docs/` | [Default community health files](https://docs.github.com/en/communities/setting-up-your-project-for-healthy-contributions/creating-a-default-community-health-file) |
| GOVERNANCE.md | Yes | Root, `.github/`, or `docs/` | [Default community health files](https://docs.github.com/en/communities/setting-up-your-project-for-healthy-contributions/creating-a-default-community-health-file) |
| FUNDING.yml | Yes | `.github/FUNDING.yml` | [Displaying a sponsor button](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/displaying-a-sponsor-button-in-your-repository) |
| Org-level defaults | N/A | Public `.github` repo in org | [Default community health files](https://docs.github.com/en/communities/setting-up-your-project-for-healthy-contributions/creating-a-default-community-health-file) |

---

## 6. Development Environment

| Item | Copies? | Configuration File | Docs |
|------|---------|-------------------|------|
| Dev container config | Yes | `.devcontainer/devcontainer.json` | [Introduction to dev containers](https://docs.github.com/en/codespaces/setting-up-your-project-for-codespaces/adding-a-dev-container-configuration/introduction-to-dev-containers) |
| Dev container directory | Yes | `.devcontainer/` | [Adding a dev container configuration](https://docs.github.com/en/codespaces/setting-up-your-project-for-codespaces/adding-a-dev-container-configuration) |
| EditorConfig | Yes | `.editorconfig` | N/A (standard file) |
| VS Code settings | Yes | `.vscode/settings.json` | N/A (standard directory) |
| VS Code extensions | Yes | `.vscode/extensions.json` | N/A (standard directory) |

---

## 7. GitHub Codespaces

| Item | Copies? | Configuration File | Docs |
|------|---------|-------------------|------|
| Dev container config | Yes | `.devcontainer/devcontainer.json` | [Template repos for Codespaces](https://docs.github.com/en/codespaces/setting-up-your-project-for-codespaces/setting-up-your-repository/setting-up-a-template-repository-for-github-codespaces) |
| Lifecycle scripts | Yes | Referenced in `devcontainer.json` | [Introduction to dev containers](https://docs.github.com/en/codespaces/setting-up-your-project-for-codespaces/adding-a-dev-container-configuration/introduction-to-dev-containers) |
| Port forwarding config | Yes | `portsAttributes` in `devcontainer.json` | [Template repos for Codespaces](https://docs.github.com/en/codespaces/setting-up-your-project-for-codespaces/setting-up-your-repository/setting-up-a-template-repository-for-github-codespaces) |
| Auto-open files | Yes | `customizations.codespaces.openFiles` in `devcontainer.json` | [Template repos for Codespaces](https://docs.github.com/en/codespaces/setting-up-your-project-for-codespaces/setting-up-your-repository/setting-up-a-template-repository-for-github-codespaces) |
| Prebuild configuration | **No** | Repository settings | [About Codespaces prebuilds](https://docs.github.com/en/codespaces/prebuilding-your-codespaces/about-github-codespaces-prebuilds) |
| Codespaces secrets | **No** | Repository/Org settings | [Managing development environment secrets](https://docs.github.com/en/codespaces/managing-codespaces-for-your-organization/managing-development-environment-secrets-for-your-repository-or-organization) |

---

## 8. GitHub Actions

| Item | Copies? | Configuration File | Docs |
|------|---------|-------------------|------|
| Workflow files | Yes | `.github/workflows/*.yml` | [Workflow syntax](https://docs.github.com/actions/using-workflows/workflow-syntax-for-github-actions) |
| Workflow permissions (in file) | Yes | `permissions:` key in workflow files | [Workflow syntax](https://docs.github.com/actions/using-workflows/workflow-syntax-for-github-actions) |
| Composite action definitions | Yes | `action.yml` | [GitHub Actions](https://docs.github.com/en/actions) |
| Actions secrets | **No** | Settings > Secrets and variables | [Using secrets in GitHub Actions](https://docs.github.com/actions/security-guides/using-secrets-in-github-actions) |
| Actions variables | **No** | Settings > Secrets and variables | [Using secrets in GitHub Actions](https://docs.github.com/actions/security-guides/using-secrets-in-github-actions) |
| Environment configurations | **No** | Settings > Environments | [Using secrets in GitHub Actions](https://docs.github.com/actions/security-guides/using-secrets-in-github-actions) |
| Default workflow permissions | **No** | Settings > Actions > General | [Managing Actions settings](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/enabling-features-for-your-repository/managing-github-actions-settings-for-a-repository) |
| OIDC trust policies | **No** | External provider config | [Configuring OIDC in AWS](https://docs.github.com/actions/deployment/security-hardening-your-deployments/configuring-openid-connect-in-amazon-web-services) |

---

## 9. GitHub Copilot

| Item | Copies? | Configuration File | Docs |
|------|---------|-------------------|------|
| Repository custom instructions | Yes | `.github/copilot-instructions.md` | [Custom instructions for Copilot](https://docs.github.com/copilot/customizing-copilot/adding-custom-instructions-for-github-copilot) |
| Path-specific instructions | Yes | `.github/instructions/*.instructions.md` | [Custom instructions for Copilot](https://docs.github.com/copilot/customizing-copilot/adding-custom-instructions-for-github-copilot) |
| Reusable prompts | Yes | `.github/prompts/*.prompt.md` | [Custom instructions for Copilot](https://docs.github.com/copilot/customizing-copilot/adding-custom-instructions-for-github-copilot) |
| Agent instructions | Yes | `AGENTS.md` (anywhere in repo) | [Custom instructions for Copilot](https://docs.github.com/copilot/customizing-copilot/adding-custom-instructions-for-github-copilot) |
| Claude Code instructions | Yes | `CLAUDE.md` | [Custom instructions for Copilot](https://docs.github.com/copilot/customizing-copilot/adding-custom-instructions-for-github-copilot) |
| Copilot setup steps | Yes | `.github/workflows/copilot-setup-steps.yml` | [Customize agent environment](https://docs.github.com/en/copilot/how-tos/use-copilot-agents/coding-agent/customize-the-agent-environment) |
| "Jumpstart with Copilot" prompt | **No** | UI feature during creation only | [Creating templates](https://docs.github.com/en/copilot/tutorials/copilot-chat-cookbook/communicate-effectively/creating-templates) |

---

## 10. Integrations and Apps

| Item | Copies? | Configuration File | Docs |
|------|---------|-------------------|------|
| GitHub App installations | **No** | Can select during creation | [Installing GitHub Apps](https://docs.github.com/en/apps/using-github-apps/installing-a-github-app-from-github-marketplace-for-your-organizations) |
| Webhook configurations | **No** | Must reconfigure manually | [Creating webhooks](https://docs.github.com/en/webhooks/using-webhooks/creating-webhooks) |
| Probot settings | Yes (file) | `.github/settings.yml` | [Probot settings](https://probot.github.io/apps/settings/) |

---

## 11. Labels and Milestones

| Item | Copies? | Configuration File | Docs |
|------|---------|-------------------|------|
| Repository labels | **No** | `.github/settings.yml` (probot) or `labels.json` (label-sync) | [Managing labels](https://docs.github.com/en/issues/using-labels-and-milestones-to-track-work/managing-labels) |
| Milestones | **No** | `.github/settings.yml` (probot) | [About milestones](https://docs.github.com/en/issues/using-labels-and-milestones-to-track-work/about-milestones) |
| Default labels | **No** | New repo gets GitHub's built-in defaults | [Managing labels](https://docs.github.com/en/issues/using-labels-and-milestones-to-track-work/managing-labels) |

> **Workaround:** Include a `.github/settings.yml` for [Probot settings](https://probot.github.io/apps/settings/) or a `labels.json` for [github-label-sync](https://github.com/Financial-Times/github-label-sync).

---

## 12. GitHub Pages

| Item | Copies? | Configuration File | Docs |
|------|---------|-------------------|------|
| Pages settings (enabled/source) | **No** | Must enable in Settings > Pages | [Configuring publishing source](https://docs.github.com/en/pages/getting-started-with-github-pages/configuring-a-publishing-source-for-your-github-pages-site) |
| gh-pages branch | Optional | Included if "Include all branches" selected | [Creating a repository from a template](https://docs.github.com/en/repositories/creating-and-managing-repositories/creating-a-repository-from-a-template) |
| CNAME file | Yes (file) | `CNAME` in repo root or gh-pages branch | [Managing a custom domain](https://docs.github.com/en/pages/configuring-a-custom-domain-for-your-github-pages-site/managing-a-custom-domain-for-your-github-pages-site) |
| Custom domain settings | **No** | Repository settings | [Managing a custom domain](https://docs.github.com/en/pages/configuring-a-custom-domain-for-your-github-pages-site/managing-a-custom-domain-for-your-github-pages-site) |

---

## 13. Packages and Releases

| Item | Copies? | Configuration File | Docs |
|------|---------|-------------------|------|
| GitHub Packages | **No** | N/A | [Introduction to GitHub Packages](https://docs.github.com/en/packages/learn-github-packages/introduction-to-github-packages) |
| Releases | **No** | N/A | [About releases](https://docs.github.com/en/repositories/releasing-projects-on-github/about-releases) |
| Git tags | **No** | N/A — history doesn't copy | [Viewing releases and tags](https://docs.github.com/en/repositories/releasing-projects-on-github/viewing-your-repositorys-releases-and-tags) |
| Container images | **No** | N/A | [Working with Container registry](https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-container-registry) |
| Release assets/binaries | **No** | N/A | [About releases](https://docs.github.com/en/repositories/releasing-projects-on-github/about-releases) |

---

## What Does NOT Copy — Complete Summary

| Item | Category | Workaround |
|------|----------|------------|
| Git commit history | Core Git | None — templates start fresh |
| Git tags | Core Git | Recreate manually or via automation |
| Git LFS files | Core Git | None — template restriction |
| Branch relationships | Core Git | Branches have unrelated histories |
| Stars / Watchers / Forks | Social | None — starts at zero |
| Issues | Project Management | Migrate via GitHub API |
| Pull requests | Project Management | Migrate via GitHub API |
| Discussions | Project Management | Recreate manually |
| Projects (classic & new) | Project Management | Use project templates separately |
| Wiki pages | Documentation | Clone wiki git repo separately |
| Releases | Distribution | Recreate after establishing tags |
| GitHub Packages | Distribution | Republish |
| Container images | Distribution | Rebuild and push |
| Labels | Issue Tracking | `.github/settings.yml` + probot or `labels.json` + label-sync |
| Milestones | Issue Tracking | `.github/settings.yml` + probot |
| Branch protection rules | Security | Org-level rulesets or export/import JSON |
| Repository rulesets | Security | Export/import JSON |
| Security feature toggles | Security | Enable via API or org-level defaults |
| Webhooks | Integrations | Reconfigure manually or via API |
| GitHub App installations | Integrations | Select during creation or install after |
| Actions secrets | CI/CD | Reconfigure in Settings > Secrets |
| Actions variables | CI/CD | Reconfigure in Settings > Secrets |
| Environments | CI/CD | Reconfigure in Settings > Environments |
| Deploy keys | CI/CD | Regenerate and reconfigure |
| OIDC trust policies | CI/CD | Reconfigure in cloud provider |
| Pages settings | Publishing | Enable in Settings > Pages |
| Custom domain (Pages) | Publishing | CNAME file copies; settings don't |
| Prebuild configuration | Codespaces | Reconfigure in repo settings |
| Codespaces secrets | Codespaces | Reconfigure in repo/org settings |
| Collaborators / Teams | Permissions | Add manually or inherit from org |
| Default workflow permissions | Settings | Reconfigure in Settings > Actions |
| Merge strategy settings | Settings | `.github/settings.yml` + probot |
| Repository topics | Metadata | Add manually or via API |
| Social preview image | Metadata | Upload in Settings |
| About description | Metadata | Set manually |
| "Jumpstart" Copilot prompt | Copilot | UI-only feature during creation |

---

## The Rule

> **Files copy. Settings don't.**
>
> If it lives in a file committed to the repository, it copies. If it's a toggle, secret, or configuration stored in GitHub's UI/API, it does not.
