from fastapi import APIRouter
from pydantic import BaseModel, Field
from app.local_engine import grounded

router = APIRouter(prefix="/copilot", tags=["Grounded retrieval"])
class Evidence(BaseModel):
    id: str = Field(max_length=100)
    organization_id: str = Field(max_length=100)
    title: str = Field(max_length=200)
    content: str = Field(max_length=100000)
    permitted: bool = False
    stale: bool = True
class GroundedRequest(BaseModel):
    organization_id: str = Field(max_length=100)
    question: str = Field(min_length=1, max_length=20000)
    evidence: list[Evidence] = Field(default_factory=list, max_length=20)
@router.post("/grounded")
async def draft(payload: GroundedRequest):
    return grounded(payload.organization_id, payload.question, [e.model_dump() for e in payload.evidence])
