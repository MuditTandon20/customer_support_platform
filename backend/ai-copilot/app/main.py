import logging
import os
from contextlib import asynccontextmanager
from dotenv import load_dotenv
from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

# Load environment variables from .env file
load_dotenv()

from app import __version__
from app.llm_client import llm_client
from app.routers import copilot
from app.schemas import HealthResponse

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(levelname)s] %(name)s: %(message)s",
)
logger = logging.getLogger("ai_copilot")


@asynccontextmanager
async def lifespan(app: FastAPI):
    logger.info("Starting AI Copilot Service v%s...", __version__)
    if llm_client.is_configured:
        logger.info("LLM Client is active with model '%s'", llm_client.model)
    else:
        logger.warning(
            "LLM Client is running in MOCK mode. Configure OPENAI_API_KEY in .env to enable real LLM requests."
        )
    yield
    logger.info("Shutting down AI Copilot Service...")


app = FastAPI(
    title="Customer Support AI Copilot API",
    description="Microservice providing AI-powered ticket summarization, smart categorization, and response drafting.",
    version=__version__,
    lifespan=lifespan,
)

# CORS middleware for frontend / microservices interaction
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Register Copilot Router
app.include_router(copilot.router)


@app.get("/", tags=["Root"])
async def root():
    return {
        "service": "AI Copilot Service",
        "version": __version__,
        "docs_url": "/docs",
        "health_url": "/health",
    }


@app.get("/health", response_model=HealthResponse, tags=["Health"])
async def health_check() -> HealthResponse:
    return HealthResponse(
        status="healthy",
        version=__version__,
        llm_configured=llm_client.is_configured,
        model=llm_client.model,
    )


if __name__ == "__main__":
    import uvicorn

    host = os.getenv("HOST", "0.0.0.0")
    port = int(os.getenv("PORT", "8001"))
    uvicorn.run("app.main:app", host=host, port=port, reload=True)
