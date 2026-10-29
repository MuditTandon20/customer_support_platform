"""Deterministic, non-generative analysis. Input is evidence, never authorization."""
import re

def summarize(subject: str, content: str) -> dict:
    sentences = re.split(r"(?<=[.!?])\s+|\n+", content.strip())
    points = [s.strip()[:300] for s in sentences if s.strip()][:4]
    return {"summary": (subject + (": " if points else "") + " ".join(points))[:1200],
            "key_points": points, "detected_sentiment": None, "mode": "deterministic_extract"}

def categorize(subject: str, content: str) -> dict:
    text = f"{subject} {content}".lower()
    rules = {"Billing": ("invoice", "payment", "refund", "billing"),
             "Account": ("login", "password", "account", "sign in"),
             "Technical": ("error", "bug", "crash", "outage")}
    scores = {category: sum(bool(re.search(r"\b"+re.escape(word)+r"\b", text)) for word in words)
              for category, words in rules.items()}
    best = max(scores, key=scores.get)
    category = best if scores[best] else "General"
    urgent = bool(re.search(r"\b(outage|urgent|production down)\b", text))
    return {"category": category, "confidence": 0.0, "priority": "HIGH" if urgent else "MEDIUM",
            "tags": [category.lower()], "reasoning": "Local keyword rule; confidence is not calibrated. Human review required.",
            "mode": "deterministic_rule"}

def grounded(organization_id: str, question: str, evidence: list[dict]) -> dict:
    # Organization scope is supplied only by the authenticated Rust service.
    words = set(re.findall(r"\w+", question.lower())) - {"the", "and", "a", "is", "how", "to", "my"}
    permitted = [e for e in evidence if e.get("organization_id") == organization_id
                 and e.get("permitted") is True and not e.get("stale", True)
                 and words.intersection(re.findall(r"\w+", (e.get("title", "")+" "+e.get("content", "")).lower()))]
    citations = [{"id": e["id"], "title": e["title"], "excerpt": e["content"][:1200]} for e in permitted[:3]]
    return {"draft_reply": "\n\n".join(f"[{i+1}] {c['title']}\n{c['excerpt']}" for i, c in enumerate(citations))
            if citations else "Insufficient permitted, current evidence. Escalate to a human.",
            "citations": citations, "abstained": not citations, "mode": "extractive", "tool_calls": []}
