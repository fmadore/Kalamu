# Repository statistics

GitHub shows a release's downloads only as a running total, and the visits to a repository
(Insights → Traffic) only for the last 14 days. The daily
[Repository statistics](../.github/workflows/stats.yml) workflow writes both down, so their
history is kept from the first run onwards. Nothing older can be recovered.

## Where to read them

On the **`stats`** branch, which the first run creates:

- `README.md`, which GitHub shows on the branch's page: downloads per version (total, last 7
  and 30 days), downloads per day, the files of the latest version, and views and unique
  visitors per day for the last 60 days.
- `data/`, CSV files for a spreadsheet or a dashboard:

| File | One row per | Columns |
| --- | --- | --- |
| `releases.csv` | day and release | `date`, `tag`, `published`, `prerelease`, `downloads` (that release's total that day) |
| `assets.csv` | change of a file's total | `date`, `tag`, `asset`, `downloads`: a file's total on any day is its latest row up to that day |
| `views.csv` | day | `date`, `views`, `unique_visitors` (GitHub's figures, UTC) |

Downloads on a day are the difference between two days' totals. Unique visitors cannot be added
up across days: a person who visits on two days counts on each.

## Setting it up

Downloads need nothing: the workflow's own token reads releases. Visits need a token GitHub
Actions cannot have by itself:

1. In GitHub, **Settings → Developer settings → Personal access tokens → Fine-grained tokens →
   Generate new token**. Choose this repository (or both Kalamu and Live Translation, to use one
   token for the two), and under **Repository permissions** set **Administration** to
   **Read-only**; nothing else. Give it an expiry date and note it.
2. In this repository, **Settings → Secrets and variables → Actions → New repository secret**,
   named `STATS_TOKEN`, with the token as its value.
3. **Actions → Repository statistics → Run workflow** records the first day straight away.

Without the secret, downloads are still recorded and the run shows a warning. When the token
expires, the run fails, and GitHub sends its usual e-mail: make a new token and replace the
secret. Days missed in between come back for visits, which GitHub keeps for 14 days, but not for
downloads.

The workflow runs every day at 04:17 UTC. GitHub stops scheduled workflows in a repository
without any activity for 60 days; **Run workflow** starts it again.
