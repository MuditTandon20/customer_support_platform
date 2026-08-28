# 🚀 Customer Support Automation Platform

An end-to-end, high-performance Customer Support and Automation Platform built with a multi-service architecture: **Rust (Axum)** for the high-throughput core API, **Python (FastAPI)** for the intelligent AI Copilot microservice, and **Next.js (React & Tailwind CSS)** for the modern frontend.

---

## 🌟 Features

- **🏢 Multi-Tenant Architecture:** Built-in organization isolation and role-based access control (RBAC).
- **⚡ High-Performance Core API:** Rust Axum backend with SQLx and PostgreSQL for ultra-fast ticket routing and transaction safety.
- **🤖 AI-Powered Copilot:** Python FastAPI service providing automated ticket triage, sentiment analysis, smart summarization, and suggested response drafting.
- **🔄 Transactional Outbox Pattern:** Guarantees reliable asynchronous event delivery and integration between microservices.
- **🎨 Modern Web UI:** Next.js App Router, TypeScript, and clean responsive UI.

---

## 🛠️ Architecture & Tech Stack

| Layer | Technology | Description |
|---|---|---|
| **Frontend** | Next.js 15, React 19, TypeScript, Tailwind CSS | Responsive dashboard & customer portal |
| **Core API** | Rust, Axum, SQLx, Tokio, PostgreSQL | Authentication, ticket management, event outbox |
| **AI Copilot** | Python 3.10+, FastAPI, OpenAI / LLM API | Auto-summarization & smart response generator |
| **Database** | PostgreSQL | Relational storage for tenants, users, and tickets |
| **Infrastructure** | Docker Compose | Local containerized database & dependencies |

---

## 📂 Project Structure

```text
customer_support_platform/
├── backend/
│   ├── rust-api/              # Core API service (Axum + SQLx + Postgres)
│   │   ├── migrations/        # SQL migration scripts
│   │   ├── src/               # Rust source code (handlers, auth, outbox)
│   │   ├── Cargo.toml
│   │   └── docker-compose.yml # Postgres local setup
│   └── ai-copilot/            # AI Microservice (FastAPI + LLM)
│       ├── app/               # FastAPI application & routers
│       └── requirements.txt
├── frontend/                  # Modern Web Dashboard (Next.js)
│   ├── src/app/               # App Router pages & layouts
│   └── package.json
├── docs/                      # Project documentation & design specs
├── LICENSE                    # MIT License
└── README.md
```

---

## 🚀 Getting Started

### 1. Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) (1.75+)
- [Python](https://www.python.org/) (3.10+)
- [Node.js](https://nodejs.org/) (v18+)
- [Docker](https://www.docker.com/) (for PostgreSQL)

---

### 2. Database Setup
Start PostgreSQL using Docker:
```bash
cd backend/rust-api
docker-compose up -d
```

Run database migrations:
```bash
cargo install sqlx-cli --no-default-features --features postgres
sqlx migrate run
```

---

### 3. Running Backend (Rust API)
```bash
cd backend/rust-api

# Copy environment file
cp .env.example .env

# Run the API server
cargo run
```
*API runs on: `http://127.0.0.1:3000`*

---

### 4. Running AI Copilot (Python FastAPI)
```bash
cd backend/ai-copilot

# Create and activate virtual environment
python -m venv venv
# Windows:
.\venv\Scripts\activate
# Linux/macOS:
# source venv/bin/activate

# Install dependencies
pip install -r requirements.txt

# Copy environment file and configure OPENAI_API_KEY
cp .env.example .env

# Start FastAPI server
uvicorn app.main:app --host 0.0.0.0 --port 8001 --reload
```
*AI Copilot API runs on: `http://127.0.0.1:8001` (Docs: `http://127.0.0.1:8001/docs`)*

---

### 5. Running Frontend (Next.js)
```bash
cd frontend

# Install dependencies
npm install

# Copy environment file
cp .env.example .env.local

# Run Next.js development server
npm run dev
```
*Frontend runs on: `http://localhost:3000`*

---

## 🛡️ License

This project is licensed under the [MIT License](LICENSE).
