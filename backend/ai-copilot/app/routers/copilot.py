from fastapi import APIRouter, HTTPException, status
from app.llm_client import llm_client
from app.schemas import (
    CategorizeRequest,
    CategorizeResponse,
    DraftReplyRequest,
    DraftReplyResponse,
    SummarizeRequest,
    SummarizeResponse,
)

router = APIRouter(
    prefix="/copilot",
    tags=["AI Copilot"],
)


@router.post(
    "/summarize",
    response_model=SummarizeResponse,
    summary="Summarize Ticket",
    description="Generate a concise summary, key points, and customer sentiment from ticket description or message history.",
)
async def summarize_ticket_endpoint(payload: SummarizeRequest) -> SummarizeResponse:
    try:
        messages_dict = (
            [m.model_dump() for m in payload.messages] if payload.messages else None
        )
        result = await llm_client.summarize_ticket(
            subject=payload.subject,
            content=payload.content,
            messages=messages_dict,
            max_length=payload.max_length,
        )

        return SummarizeResponse(
            ticket_id=payload.ticket_id,
            summary=result["summary"],
            key_points=result.get("key_points", []),
            detected_sentiment=result.get("detected_sentiment"),
        )
    except Exception as e:
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail=f"Failed to summarize ticket: {str(e)}",
        )


@router.post(
    "/categorize",
    response_model=CategorizeResponse,
    summary="Categorize Ticket",
    description="Classify a ticket into domain category, priority, and suggest tags with confidence score.",
)
async def categorize_ticket_endpoint(payload: CategorizeRequest) -> CategorizeResponse:
    try:
        result = await llm_client.categorize_ticket(
            subject=payload.subject,
            content=payload.content,
            candidate_categories=payload.candidate_categories,
        )

        return CategorizeResponse(
            ticket_id=payload.ticket_id,
            category=result["category"],
            confidence=result["confidence"],
            priority=result["priority"],
            tags=result.get("tags", []),
            reasoning=result.get("reasoning"),
        )
    except Exception as e:
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail=f"Failed to categorize ticket: {str(e)}",
        )


@router.post(
    "/draft-reply",
    response_model=DraftReplyResponse,
    summary="Draft Support Reply",
    description="Generate an empathetic and contextual draft response for support agents to review and send.",
)
async def draft_reply_endpoint(payload: DraftReplyRequest) -> DraftReplyResponse:
    try:
        history_dict = (
            [m.model_dump() for m in payload.conversation_history]
            if payload.conversation_history
            else None
        )
        result = await llm_client.draft_reply(
            subject=payload.subject,
            content=payload.content,
            customer_name=payload.customer_name or "Customer",
            agent_name=payload.agent_name or "Support Agent",
            tone=payload.tone or "professional and empathetic",
            conversation_history=history_dict,
            additional_instructions=payload.additional_instructions,
        )

        return DraftReplyResponse(
            ticket_id=payload.ticket_id,
            draft_reply=result["draft_reply"],
            suggested_action=result.get("suggested_action"),
            tone_used=result.get("tone_used", payload.tone or "professional"),
        )
    except Exception as e:
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail=f"Failed to draft reply: {str(e)}",
        )
