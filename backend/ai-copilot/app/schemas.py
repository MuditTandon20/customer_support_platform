from typing import List, Optional
from pydantic import BaseModel, Field


class MessageItem(BaseModel):
    role: str = Field(..., description="Role of message author: 'user', 'customer', 'agent', 'assistant', 'system'")
    content: str = Field(..., description="Message text content")
    sender: Optional[str] = Field(None, description="Optional sender name or identifier")


# --- 1. Summarize Endpoint Models ---
class SummarizeRequest(BaseModel):
    ticket_id: Optional[str] = Field(None, description="Optional unique identifier for the ticket")
    subject: Optional[str] = Field(None, description="Subject of the support ticket")
    content: Optional[str] = Field(None, description="Single text body or description of the issue")
    messages: Optional[List[MessageItem]] = Field(None, description="Optional thread/chat history of the ticket")
    max_length: Optional[int] = Field(None, description="Optional maximum character/word guidance for summary")


class SummarizeResponse(BaseModel):
    ticket_id: Optional[str] = None
    summary: str = Field(..., description="Concise summary of the ticket issue and status")
    key_points: List[str] = Field(default_factory=list, description="Key bullet points extracted from the conversation")
    detected_sentiment: Optional[str] = Field(None, description="Detected customer sentiment (e.g., Frustrated, Neutral, Satisfied)")


# --- 2. Categorize Endpoint Models ---
class CategorizeRequest(BaseModel):
    ticket_id: Optional[str] = Field(None, description="Optional unique identifier for the ticket")
    subject: str = Field(..., description="Subject of the ticket")
    content: str = Field(..., description="Issue description or message body")
    candidate_categories: Optional[List[str]] = Field(
        None,
        description="Optional list of predefined categories to classify into"
    )


class CategorizeResponse(BaseModel):
    ticket_id: Optional[str] = None
    category: str = Field(..., description="Primary categorized topic/department")
    confidence: float = Field(..., ge=0.0, le=1.0, description="Confidence score between 0.0 and 1.0")
    priority: str = Field(..., description="Suggested ticket priority (e.g. LOW, MEDIUM, HIGH, URGENT)")
    tags: List[str] = Field(default_factory=list, description="Descriptive labels or tags")
    reasoning: Optional[str] = Field(None, description="Brief explanation for the assigned category and priority")


# --- 3. Draft Reply Endpoint Models ---
class DraftReplyRequest(BaseModel):
    ticket_id: Optional[str] = Field(None, description="Optional unique identifier for the ticket")
    subject: Optional[str] = Field(None, description="Ticket subject line")
    content: str = Field(..., description="Customer's latest inquiry or full ticket message")
    customer_name: Optional[str] = Field("Customer", description="Name of the customer to address")
    agent_name: Optional[str] = Field("Support Team", description="Name/signature of the support agent")
    conversation_history: Optional[List[MessageItem]] = Field(None, description="Previous messages in the conversation")
    tone: Optional[str] = Field("professional, helpful, and empathetic", description="Desired tone of voice")
    additional_instructions: Optional[str] = Field(
        None,
        description="Extra context, internal notes, resolution steps, or policy constraints"
    )


class DraftReplyResponse(BaseModel):
    ticket_id: Optional[str] = None
    draft_reply: str = Field(..., description="Generated draft response ready for the agent to review and send")
    suggested_action: Optional[str] = Field(None, description="Recommended next action for the agent (e.g., request logs, escalate)")
    tone_used: str = Field(..., description="The tone applied in generating the reply")


# --- Health check ---
class HealthResponse(BaseModel):
    status: str
    version: str
    llm_configured: bool
    model: str
