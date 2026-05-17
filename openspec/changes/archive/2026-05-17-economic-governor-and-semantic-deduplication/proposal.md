# Proposal: Economic Governor and Semantic Deduplication (FinOps)

## Context
Nebula's Active Learning loop depends on a high-parameter "Teacher" model (Tier 3) to generate corrections (SFT) and preferences (DPO). Since Tier 3 relies on external, token-billed APIs, an unmitigated spike in hallucinations (e.g., due to a trending topic or a bot scraping the swarm) could result in catastrophic financial costs. To ensure enterprise viability, Nebula must implement strict FinOps controls to prevent duplicate API calls and enforce tenant-level budget caps.

## Objectives
Implement a Cost-Control Gateway:
1. **Semantic Deduplicator (`nebula-semantic-deduplicator`)**: A FaaS that intercepts divergence alerts and uses fast, local embeddings to check if a conceptually identical hallucination is already in the Teacher's queue or recently resolved.
2. **Economic Governor (`nebula-economic-governor`)**: A strict rate-limiter and budget tracker that counts Tier 3 token usage per `x-tenant-id` and halts arbitration requests if a daily/monthly budget is exceeded.
3. **FinOps Dashboard (VS Code)**: A UI panel for administrators to set API quotas, monitor token burn rates, and view the deduplication savings.