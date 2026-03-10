---
diataxis_type: reference
---
# Metrics & Observability Dashboard

## Overview

Track project health, downloads, performance, and usage metrics.

## Available Metrics

### 1. GitHub Insights

**Built-in GitHub metrics:**
- **Stars/Forks/Watchers** - Popularity indicators
- **Issues/PRs** - Activity metrics
- **Contributors** - Community growth
- **Traffic** - Views and clones
- **Releases** - Download counts

**Access:** Repository → Insights tab

### 2. crates.io Statistics

**Package metrics:**
- **Downloads** - Total and recent downloads
- **Versions** - Release history
- **Dependencies** - Reverse dependency count

**API Access:**
```bash
# Get crate info
curl https://crates.io/api/v1/crates/nsip

# Get download stats
curl https://crates.io/api/v1/crates/nsip/downloads
```

**Response:**
```json
{
  "crate": {
    "id": "nsip",
    "name": "nsip",
    "downloads": 12345,
    "recent_downloads": 1234
  },
  "versions": [
    {
      "num": "0.1.0",
      "downloads": 5678
    }
  ]
}
```

### 3. Docker Hub Statistics

**Container metrics:**
- **Pulls** - Image download count
- **Stars** - Repository stars

**API Access:**
```bash
curl https://hub.docker.com/v2/repositories/username/nsip/

# Pull statistics
curl https://hub.docker.com/v2/repositories/username/nsip/stats/
```

### 4. GitHub Packages

**Package registry metrics:**
```bash
gh api /users/USERNAME/packages/container/nsip
```

### 5. CI/CD Metrics

**Workflow metrics:**
- **Build times** - Performance tracking
- **Success rate** - Reliability metric
- **Cache hit rate** - Efficiency metric

**Query via GitHub API:**
```bash
gh api repos/USER/REPO/actions/workflows/ci.yml/runs \
  --jq '.workflow_runs[] | {created_at, conclusion, run_duration_ms}'
```

## Dashboard Options

### Option 1: shields.io Badges

**Add to README.md:**

```markdown
<!-- Build Status -->
![CI](https://github.com/USER/REPO/workflows/CI/badge.svg)

<!-- Coverage -->
[![codecov](https://codecov.io/gh/USER/REPO/branch/main/graph/badge.svg)](https://codecov.io/gh/USER/REPO)

<!-- crates.io -->
[![Crates.io](https://img.shields.io/crates/v/nsip)](https://crates.io/crates/nsip)
[![Downloads](https://img.shields.io/crates/d/nsip)](https://crates.io/crates/nsip)

<!-- Docker -->
[![Docker Pulls](https://img.shields.io/docker/pulls/username/nsip)](https://hub.docker.com/r/username/nsip)

<!-- Dependencies -->
[![Deps.rs](https://deps.rs/repo/github/USER/REPO/status.svg)](https://deps.rs/repo/github/USER/REPO)

<!-- Security -->
[![Security Audit](https://github.com/USER/REPO/workflows/Security%20Audit/badge.svg)](https://github.com/USER/REPO/actions?query=workflow%3A%22Security+Audit%22)
```

### Option 2: GitHub Metrics Action

**`.github/workflows/metrics.yml`:**

```yaml
name: Metrics
on:
  schedule:
    - cron: "0 0 * * 0"  # Weekly
  workflow_dispatch:

jobs:
  metrics:
    runs-on: ubuntu-latest
    permissions:
      contents: write
    steps:
      - uses: lowlighter/metrics@latest
        with:
          token: ${{ secrets.METRICS_TOKEN }}
          user: username
          template: classic
          config_timezone: America/New_York
          plugin_lines: yes
          plugin_languages: yes
          plugin_languages_details: percentage
          plugin_traffic: yes
          plugin_stargazers: yes
          plugin_stargazers_charts_type: chartist
```

**Generates:** SVG metrics dashboard

### Option 3: Custom Dashboard

**Grafana + Prometheus:**

```yaml
# docker-compose.yml
version: '3'
services:
  prometheus:
    image: prom/prometheus
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
    ports:
      - 9090:9090

  grafana:
    image: grafana/grafana
    ports:
      - 3000:3000
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
```

**prometheus.yml:**
```yaml
scrape_configs:
  - job_name: 'github'
    static_configs:
      - targets: ['api.github.com']
    metrics_path: '/repos/USER/REPO'
```

### Option 4: Plausible Analytics

For documentation site traffic:

```html
<!-- docs/index.html -->
<script defer data-domain="yourdomain.com" src="https://plausible.io/js/script.js"></script>
```

**Features:**
- Privacy-friendly
- No cookies
- GDPR compliant
- Real-time dashboard

## Performance Benchmarks

### Criterion Integration

**View Benchmark Reports:**

1. Run benchmarks locally: `cargo bench`
2. Open `target/criterion/report/index.html`
3. Compare with baseline

**CI Reports:**
- Saved to Actions artifacts
- Compare across PRs
- Track performance trends

### Continuous Benchmarking

**Track performance over time:**

```bash
# Generate benchmark history
cargo bench --bench my_benchmark -- --save-baseline current

# Compare with previous
cargo bench --bench my_benchmark -- --baseline previous
```

**Store results:**
```yaml
- name: Upload benchmark results
  uses: actions/upload-artifact@v4
  with:
    name: benchmarks-${{ github.sha }}
    path: target/criterion/
    retention-days: 90
```

## Download Tracking

### crates.io Downloads

**Script to track:**

```python
import requests
import json
from datetime import datetime

def get_crate_stats(crate_name):
    url = f"https://crates.io/api/v1/crates/{crate_name}"
    response = requests.get(url)
    data = response.json()

    return {
        "date": datetime.now().isoformat(),
        "total_downloads": data["crate"]["downloads"],
        "recent_downloads": data["crate"]["recent_downloads"],
        "versions": len(data["versions"])
    }

# Store in time-series database or CSV
stats = get_crate_stats("nsip")
with open("downloads.json", "a") as f:
    json.dump(stats, f)
    f.write("\n")
```

**Automate with GitHub Actions:**

```yaml
name: Track Downloads
on:
  schedule:
    - cron: '0 0 * * *'  # Daily

jobs:
  track:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Fetch stats
        run: python scripts/track_downloads.py
      - name: Commit stats
        run: |
          git add downloads.json
          git commit -m "Update download stats [skip ci]"
          git push
```

### Visualization

**Generate charts:**

```python
import matplotlib.pyplot as plt
import json

# Load data
with open("downloads.json") as f:
    data = [json.loads(line) for line in f]

dates = [d["date"] for d in data]
downloads = [d["total_downloads"] for d in data]

# Plot
plt.plot(dates, downloads)
plt.title("Download Growth")
plt.xlabel("Date")
plt.ylabel("Total Downloads")
plt.xticks(rotation=45)
plt.tight_layout()
plt.savefig("downloads.png")
```

## Health Checks

### Repository Health Score

**GitHub Community Standards:**
- ✅ README
- ✅ LICENSE
- ✅ CODE_OF_CONDUCT.md
- ✅ CONTRIBUTING.md
- ✅ SECURITY.md
- ✅ Issue templates
- ✅ PR templates

**Check:** Repository → Insights → Community

### Dependency Health

**deps.rs Dashboard:**
- https://deps.rs/repo/github/USER/REPO

**Shows:**
- Outdated dependencies
- Security vulnerabilities
- License compatibility

### Code Quality Metrics

**From workflows:**
- Test coverage (≥80%)
- Mutation score (≥80%)
- Clippy warnings (0)
- Documentation coverage

## Alerting

### GitHub Notifications

**Configure alerts:**

1. Repository → Settings → Notifications
2. Enable:
   - Dependabot alerts
   - Secret scanning
   - Code scanning

### Custom Alerts

**Workflow failures:**

```yaml
- name: Notify on failure
  if: failure()
  uses: 8398a7/action-slack@v3
  with:
    status: ${{ job.status }}
    text: 'Build failed!'
    webhook_url: ${{ secrets.SLACK_WEBHOOK }}
```

**Performance regression:**

```yaml
- name: Check regression
  run: |
    if [ "$PERF_CHANGE" -gt "10" ]; then
      echo "Performance regression detected!"
      gh issue create --title "Performance Regression" --body "..."
    fi
```

## Reporting

### Weekly Summary

**Automated report:**

```yaml
name: Weekly Report
on:
  schedule:
    - cron: '0 9 * * 1'  # Monday 9 AM

jobs:
  report:
    runs-on: ubuntu-latest
    steps:
      - name: Generate report
        run: |
          cat > report.md << 'EOF'
          # Weekly Report

          ## Downloads
          $(curl -s https://crates.io/api/v1/crates/nsip | jq -r '.crate | "Total: \(.downloads), Recent: \(.recent_downloads)"')

          ## Issues
          Open: $(gh issue list --state open --json id | jq '. | length')
          Closed this week: $(gh issue list --state closed --search "closed:>=$(date -d '7 days ago' +%Y-%m-%d)" --json id | jq '. | length')

          ## PRs
          Merged this week: $(gh pr list --state merged --search "merged:>=$(date -d '7 days ago' +%Y-%m-%d)" --json id | jq '. | length')
          EOF

      - name: Post to discussions
        run: gh discussion create --title "Weekly Report $(date +%Y-%m-%d)" --body-file report.md
```

## Links

- [GitHub Insights](https://docs.github.com/en/repositories/viewing-activity-and-data-for-your-repository/viewing-traffic-to-a-repository)
- [crates.io API](https://crates.io/data-access)
- [Docker Hub API](https://docs.docker.com/registry/spec/api/)
- [shields.io](https://shields.io/)
- [Grafana](https://grafana.com/)
- [Plausible Analytics](https://plausible.io/)
