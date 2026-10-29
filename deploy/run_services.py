"""Supervise the retained Rust/Python processes in one free web instance.

All application/workflow state is stored in Postgres, never in this supervisor.
"""
import os
import signal
import subprocess
import sys
import threading

def main():
    for name in ('DATABASE_URL','JWT_SECRET','AI_SERVICE_TOKEN','FRONTEND_ORIGIN'):
        if not os.getenv(name):raise SystemExit(f'{name} must be configured')
    stop=threading.Event()
    for sig in (signal.SIGINT,signal.SIGTERM):signal.signal(sig,lambda *_:stop.set())
    children=[]
    try:
        children.append(subprocess.Popen([sys.executable,'-m','uvicorn','app.main:app','--host','127.0.0.1','--port','8001','--workers','1']))
        children.append(subprocess.Popen([os.getenv('RUST_API_BINARY','/usr/local/bin/rust-api')]))
        while not stop.wait(.25):
            if any(child.poll() is not None for child in children):
                print('A service exited; stopping its peer for a clean host restart.',file=sys.stderr)
                return 1
        return 0
    finally:
        for child in children:
            if child.poll() is None:child.terminate()
        for child in children:
            try:child.wait(timeout=20)
            except subprocess.TimeoutExpired:child.kill();child.wait()

if __name__=='__main__':sys.exit(main())
