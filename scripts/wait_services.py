import time
import httpx
urls=['http://localhost:18080/ready','http://localhost:18001/health','http://localhost:13000/api/health']
for url in urls:
    for _ in range(90):
        try:
            if httpx.get(url,timeout=5).status_code==200:break
        except httpx.HTTPError:pass
        time.sleep(1)
    else:raise SystemExit('Service failed readiness check: '+url)
print('Native integration services ready')
