## Code Exploration Protocol

1. **Never** perform recursive file searches, broad directory listings, or raw text grep sweeps for code discovery.
2. **Mandatory:** Use CodeGraph MCP tool (`codegraph_explore`) for all symbol lookup, dependency tracking, and architectural navigation.
3. Raw file reads are restricted strictly to inspecting the exact file contents identified via CodeGraph context queries.
