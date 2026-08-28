import json
import logging
import os
from typing import Any, Dict, List, Optional
from openai import AsyncOpenAI

logger = logging.getLogger(__name__)


class LLMClient:
    def __init__(self):
        self.api_key = os.getenv("OPENAI_API_KEY", "")
        self.base_url = os.getenv("OPENAI_BASE_URL", "https://api.openai.com/v1")
        self.model = os.getenv("LLM_MODEL", "gpt-4o-mini")
        self.temperature = float(os.getenv("LLM_TEMPERATURE", "0.2"))

        # Check if a valid API key is present
        self.is_configured = bool(
            self.api_key and self.api_key not in ("your-api-key-here", "sk-...", "")
        )

        if self.is_configured:
            self.client = AsyncOpenAI(api_key=self.api_key, base_url=self.base_url)
            logger.info("LLMClient initialized with model: %s", self.model)
        else:
            self.client = None
            logger.warning("LLMClient running in MOCK/FALLBACK mode: OPENAI_API_KEY is not configured.")

    async def _call_chat_completion(
        self,
        system_prompt: str,
        user_prompt: str,
        response_json: bool = False,
        temperature: Optional[float] = None,
    ) -> str:
        """Helper to invoke chat completion with error handling."""
        if not self.is_configured or self.client is None:
            raise RuntimeError("LLM client is not configured with an API key.")

        messages = [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": user_prompt},
        ]

        kwargs: Dict[str, Any] = {
            "model": self.model,
            "messages": messages,
            "temperature": temperature if temperature is not None else self.temperature,
        }
        if response_json:
            kwargs["response_format"] = {"type": "json_object"}

        response = await self.client.chat.completions.create(**kwargs)
        return response.choices[0].message.content or ""

    # ==========================================
    # 1. Summarize
    # ==========================================
    async def summarize_ticket(
        self,
        subject: Optional[str],
        content: Optional[str],
        messages: Optional[List[Dict[str, Any]]],
        max_length: Optional[int] = None,
    ) -> Dict[str, Any]:
        """Generate a concise summary, key points, and detected sentiment."""
        # Compile the discussion body
        text_parts = []
        if subject:
            text_parts.append(f"Subject: {subject}")
        if content:
            text_parts.append(f"Description:\n{content}")
        if messages:
            text_parts.append("Thread History:")
            for msg in messages:
                role = msg.get("role", "user")
                sender = msg.get("sender")
                prefix = f"[{sender} ({role})]" if sender else f"[{role}]"
                text_parts.append(f"{prefix}: {msg.get('content', '')}")

        full_ticket_text = "\n\n".join(text_parts).strip()
        if not full_ticket_text:
            full_ticket_text = "No content provided."

        if not self.is_configured:
            # Fallback mock response for testing/development
            return {
                "summary": f"Customer reported an issue regarding: {subject or 'Service inquiry'}. Key details extracted from initial conversation.",
                "key_points": [
                    "Initial request received from customer",
                    "Awaiting technical investigation or agent review",
                ],
                "detected_sentiment": "Neutral",
            }

        system_prompt = (
            "You are an expert customer service AI assistant. Analyze the support ticket text and return a JSON object with:\n"
            "- 'summary': A concise, factual summary (1-3 sentences) of the user's issue and current state.\n"
            "- 'key_points': A list of string bullets highlighting critical facts, errors, or customer questions.\n"
            "- 'detected_sentiment': One of ['Frustrated', 'Angry', 'Urgent', 'Neutral', 'Satisfied', 'Confused']."
        )
        if max_length:
            system_prompt += f"\nKeep the summary under roughly {max_length} characters."

        user_prompt = f"Support Ticket Data:\n\n{full_ticket_text}"

        try:
            raw_json = await self._call_chat_completion(system_prompt, user_prompt, response_json=True)
            data = json.loads(raw_json)
            return {
                "summary": data.get("summary", ""),
                "key_points": data.get("key_points", []),
                "detected_sentiment": data.get("detected_sentiment", "Neutral"),
            }
        except Exception as e:
            logger.error("Error in summarize_ticket LLM call: %s", e)
            return {
                "summary": f"Summary for '{subject or 'Ticket'}': Customer reported an issue. (LLM parsing fallback)",
                "key_points": ["Could not parse LLM output or connection issue occurred."],
                "detected_sentiment": "Neutral",
            }

    # ==========================================
    # 2. Categorize
    # ==========================================
    async def categorize_ticket(
        self,
        subject: str,
        content: str,
        candidate_categories: Optional[List[str]] = None,
    ) -> Dict[str, Any]:
        """Categorize the ticket, estimate priority, tags, and confidence."""
        default_categories = [
            "Billing & Payments",
            "Technical Support",
            "Account Access & Security",
            "Feature Request",
            "Bug Report",
            "General Inquiry",
        ]
        target_categories = candidate_categories or default_categories

        if not self.is_configured:
            # Fallback mock response for testing/development
            return {
                "category": target_categories[0] if target_categories else "General Inquiry",
                "confidence": 0.85,
                "priority": "MEDIUM",
                "tags": ["support", "automated-triage"],
                "reasoning": "Classified automatically based on keywords in ticket content (mock mode).",
            }

        system_prompt = (
            "You are an automated support ticket triage classifier. Classify the ticket into exactly one category from the allowed list.\n"
            f"Allowed Categories: {json.dumps(target_categories)}\n\n"
            "Return a JSON object with:\n"
            "- 'category': String matching one of the allowed categories (or closest match).\n"
            "- 'confidence': Float between 0.0 and 1.0 representing classification confidence.\n"
            "- 'priority': One of ['LOW', 'MEDIUM', 'HIGH', 'URGENT'].\n"
            "- 'tags': List of 2-5 relevant keyword tags.\n"
            "- 'reasoning': A brief 1-sentence explanation of why this category & priority was chosen."
        )

        user_prompt = f"Subject: {subject}\n\nContent:\n{content}"

        try:
            raw_json = await self._call_chat_completion(system_prompt, user_prompt, response_json=True)
            data = json.loads(raw_json)
            return {
                "category": data.get("category", target_categories[0]),
                "confidence": float(data.get("confidence", 0.8)),
                "priority": data.get("priority", "MEDIUM").upper(),
                "tags": data.get("tags", []),
                "reasoning": data.get("reasoning", ""),
            }
        except Exception as e:
            logger.error("Error in categorize_ticket LLM call: %s", e)
            return {
                "category": target_categories[0],
                "confidence": 0.5,
                "priority": "MEDIUM",
                "tags": ["triage"],
                "reasoning": "Fallback classification due to LLM error.",
            }

    # ==========================================
    # 3. Draft Reply
    # ==========================================
    async def draft_reply(
        self,
        subject: Optional[str],
        content: str,
        customer_name: str,
        agent_name: str,
        tone: str,
        conversation_history: Optional[List[Dict[str, Any]]] = None,
        additional_instructions: Optional[str] = None,
    ) -> Dict[str, Any]:
        """Generate a contextual, empathetic response draft for a support agent."""
        if not self.is_configured:
            # Fallback mock response for testing/development
            return {
                "draft_reply": (
                    f"Hi {customer_name},\n\n"
                    f"Thank you for reaching out regarding '{subject or 'your inquiry'}'. "
                    f"We have received your message and our team is looking into this.\n\n"
                    f"Best regards,\n{agent_name}"
                ),
                "suggested_action": "Review ticket details and confirm customer account status.",
                "tone_used": tone,
            }

        history_text = ""
        if conversation_history:
            history_lines = []
            for msg in conversation_history:
                sender = msg.get("sender") or msg.get("role", "user")
                history_lines.append(f"{sender}: {msg.get('content', '')}")
            history_text = "\nConversation History:\n" + "\n".join(history_lines)

        system_prompt = (
            f"You are a helpful customer support agent assistant named '{agent_name}'. "
            f"Draft a response to customer '{customer_name}'.\n"
            f"Tone: {tone}.\n\n"
            "Return a JSON object with:\n"
            "- 'draft_reply': The full text of the proposed email/chat response, formatted with greetings and sign-off.\n"
            "- 'suggested_action': A short note describing the next internal step the agent should take.\n"
        )

        user_prompt = (
            f"Ticket Subject: {subject or 'Customer Inquiry'}\n"
            f"Latest Customer Message:\n{content}\n"
            f"{history_text}\n"
            f"Internal Guidelines / Additional Context: {additional_instructions or 'None'}"
        )

        try:
            raw_json = await self._call_chat_completion(system_prompt, user_prompt, response_json=True)
            data = json.loads(raw_json)
            return {
                "draft_reply": data.get("draft_reply", ""),
                "suggested_action": data.get("suggested_action"),
                "tone_used": tone,
            }
        except Exception as e:
            logger.error("Error in draft_reply LLM call: %s", e)
            return {
                "draft_reply": f"Hi {customer_name},\n\nThank you for contacting us. We are currently reviewing your request regarding {subject or 'this issue'} and will get back to you shortly.\n\nBest regards,\n{agent_name}",
                "suggested_action": "Verify customer request details.",
                "tone_used": tone,
            }


# Singleton instance
llm_client = LLMClient()
