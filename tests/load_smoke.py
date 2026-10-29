"""Basic local concurrency measurement, not a production capacity claim."""
import asyncio
import json
import os
import pathlib
import statistics
import time
import uuid
import httpx

async def main():
    base=os.getenv('TEST_API_URL','http://localhost:18080')
    async with httpx.AsyncClient(base_url=base,timeout=30,headers={'Origin':os.getenv('TEST_FRONTEND_ORIGIN','http://localhost:13000')}) as c:
        r=await c.post('/auth/signup',json={'organization_name':'Load smoke '+str(uuid.uuid4()),'email':str(uuid.uuid4())+'@example.test','full_name':'Synthetic load fixture','password':'Local-load-test-2026!'})
        r.raise_for_status();headers={'Authorization':'Bearer '+r.json()['token']}
        semaphore=asyncio.Semaphore(10)
        async def one(index):
            async with semaphore:
                start=time.perf_counter()
                response=await c.get('/inbox' if index%2 else '/analytics',headers=headers)
                return {'status':response.status_code,'ms':(time.perf_counter()-start)*1000}
        start=time.perf_counter();results=await asyncio.gather(*(one(i) for i in range(100)));elapsed=time.perf_counter()-start
    latencies=sorted(r['ms'] for r in results)
    report={'requests':100,'concurrency':10,'errors':sum(r['status']!=200 for r in results),'p50_ms':round(statistics.median(latencies),2),'p95_ms':round(latencies[94],2),'elapsed_seconds':round(elapsed,2),'requests_per_second':round(100/elapsed,2),'environment':'Windows local Rust debug build; disposable PostgreSQL 18; empty synthetic tenant; 50 inbox + 50 analytics requests','scope':'Smoke measurement only. Not an availability or production SLO.'}
    pathlib.Path('docs/load-smoke.json').write_text(json.dumps(report,indent=2),encoding='utf-8')
    print(json.dumps(report,indent=2))
    if report['errors']:raise SystemExit(1)
asyncio.run(main())
