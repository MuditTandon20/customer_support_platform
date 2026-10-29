"""Fail CI when a production client JavaScript chunk exceeds the beta budget."""
from pathlib import Path
import json

root = Path(__file__).resolve().parents[1] / 'frontend' / '.next' / 'static'
chunks = sorted((p for p in root.rglob('*.js')), key=lambda p: p.stat().st_size, reverse=True)
if not chunks:
    raise SystemExit('Build the frontend before checking its client bundle.')
budget = 300_000
print(json.dumps({'single_chunk_budget_bytes': budget,
                  'all_route_chunks_bytes': sum(p.stat().st_size for p in chunks),
                  'largest_chunks': [{'path': str(p.relative_to(root)), 'bytes': p.stat().st_size}
                                     for p in chunks[:5]]}, indent=2))
if any(p.stat().st_size > budget for p in chunks):
    raise SystemExit('Client JavaScript chunk exceeds the Stage 1 budget; review splitting/dependencies.')
