# Owner documentation — 29 October 2026

| Document | Editable Word | Reading PDF | Maintained source |
|---|---|---|---|
| Stage 1 product requirements | [PRD.docx](ASTER_PRD_STAGE1.docx) | [PRD.pdf](ASTER_PRD_STAGE1.pdf) | [Markdown](../ASTER_PRD_STAGE1.md) |
| Project explained from scratch | [Guide.docx](ASTER_PROJECT_GUIDE.docx) | [Guide.pdf](ASTER_PROJECT_GUIDE.pdf) | [Markdown](../ASTER_PROJECT_GUIDE.md) |

The PRD describes the current product scope and acceptance criteria. The guide explains the project in simple language and includes a UI tour, sample credentials, architecture, safety limits and unfinished work. The [daily task record](../TASK_LOG_2026-10-29.md) saves completed work, the exact tested application revision and continuation constraints.

Dashboard: http://localhost:3000/workspace/dashboard. Sign in at http://localhost:3000/login. These links work on the machine running the local stack. No public site was deployed.

The two PNG previews were captured from a successful sign-in to the newly created synthetic local demo. They contain invented sample records. Shared demo credentials in the guide must never be reused for real customer accounts.

The application verification baseline is commit d83daf1c50dc204493889560156175d081cc5798 and [successful CI workflow #9](https://github.com/MuditTandon20/customer_support_platform/actions/runs/40900733408). This documentation handoff does not change application code or runtime dependencies.

## Regenerate

The Markdown files are the maintained source. Change them, then run scripts/export_project_documents.py with python-docx and reportlab installed in a separate documentation-tooling environment. Arial (Windows) or DejaVu Sans (Linux) is required for readable Unicode PDF text. The exporter does not start services or modify a database. It embeds the existing PNG previews if present.

Example PowerShell setup, kept outside the application environment:

```powershell
$docsTools = Join-Path $env:TEMP 'aster-document-tools'
python -m venv $docsTools
& "$docsTools\Scripts\python.exe" -m pip install python-docx reportlab
& "$docsTools\Scripts\python.exe" scripts/export_project_documents.py
```

Re-export both formats whenever the maintained source changes. Do not overwrite the user's original root-level Customer_Support_Platform_Documentation.pdf or original docs/generate_pdf.py; these are separate deliverables.
