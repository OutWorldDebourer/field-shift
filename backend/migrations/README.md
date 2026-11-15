# migrations

| File | What it does |
|------|--------------|
| `0001_init.sql` | Tables: analyses, fields (POLYGON outline), messages, photos, sections, reports, source_cache, crop_traits, rate_limits |
| `0003_steps.sql` | `analyses.steps`: state of each guided topic |
| `0004_facts.sql` | `analyses.facts`: the farmer's answers (intake form and consultant notes) |
| `0005_usage.sql` | `api_usage`: input, output, cache-read and cache-write tokens of every Anthropic call (per round, grouped by turn) |
