# Support OS architecture

```mermaid
flowchart LR
  Agent[Next.js agent console] --> API[Rust Axum / authenticated tenant context]
  Customer[Customer portal] --> API
  Widget[Embedded authenticated portal widget] --> API
  API --> DB[(Postgres / tenant-scoped rows)]
  API --> AI[FastAPI copilot / internal service]
  API --> Outbox[Transactional outbox]
  Outbox --> Kafka[Kafka-compatible broker]
  API --> Policy[Workflow and tool policy]
  Policy --> Approval[Human approval]
  Approval --> Audit[Audit / execution trace]
```

Rust owns identity, tickets, messages, knowledge permissions, proposal approval, workflow validation/execution and metrics. Retrieval only selects fresh sources in the verified organization. Browser-supplied organization IDs never authorize access. Customer routes restrict ownership and hide internal notes. Authentication checks live user role and token version. Errors omit database details.

Portal replies and internal notes commit with audit/outbox records. Provider sending is a separate capability; storing a portal reply never proves email delivery. Workflow simulation never mutates tickets. Publishing validates every graph, and runs use a saved published version. Duplicate message/run/action keys are scoped by tenant and checked against request content.

```mermaid
stateDiagram-v2
  [*] --> new
  new --> open
  open --> pending
  pending --> open
  open --> on_hold
  on_hold --> open
  open --> solved
  pending --> solved
  solved --> open
  solved --> closed
  closed --> [*]
```

See runbook.md for startup, failure handling and explicit operational limits. This is not a compliance certification.
