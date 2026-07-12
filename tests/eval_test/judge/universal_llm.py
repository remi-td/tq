"""DeepEval LLM wrapper backed by AWS Bedrock, OpenAI, OpenRouter, or Gemini."""

from __future__ import annotations

import json
import os
from typing import Any

from deepeval.models.base_model import DeepEvalBaseLLM


class UniversalLLM(DeepEvalBaseLLM):
    def __init__(self, provider: str | None = None, model_id: str | None = None):
        self.provider = (provider or os.environ.get("EVALS_PROVIDER", "bedrock")).lower()

        if self.provider == "bedrock":
            self.model_id = model_id or os.environ.get(
                "BEDROCK_JUDGE_MODEL_ID",
                os.environ.get("BEDROCK_MODEL_ID", "anthropic.claude-3-5-sonnet-20241022-v2:0")
            )
        elif self.provider == "openai":
            self.model_id = model_id or os.environ.get("OPENAI_MODEL_ID", "gpt-4o")
        elif self.provider == "openrouter":
            self.model_id = model_id or os.environ.get("OPENROUTER_MODEL_ID", "anthropic/claude-3.5-sonnet")
        elif self.provider == "gemini":
            self.model_id = model_id or os.environ.get("GEMINI_MODEL_ID", "gemini-2.5-flash")
        else:
            raise ValueError(f"Unknown provider: {self.provider}")

        self._client = self._build_client()
        super().__init__()

    def _build_client(self) -> Any:
        if self.provider == "bedrock":
            import boto3
            region = os.environ.get("AWS_REGION", "us-east-1")
            return boto3.client("bedrock-runtime", region_name=region)
        elif self.provider == "openai":
            from openai import OpenAI
            return OpenAI(api_key=os.environ.get("OPENAI_API_KEY"))
        elif self.provider == "openrouter":
            from openai import OpenAI
            return OpenAI(
                base_url="https://openrouter.ai/api/v1",
                api_key=os.environ.get("OPENROUTER_API_KEY"),
            )
        elif self.provider == "gemini":
            from openai import OpenAI
            base_url = os.environ.get("GEMINI_BASE_URL", "https://generativelanguage.googleapis.com/v1beta/openai/")
            return OpenAI(
                base_url=base_url,
                api_key=os.environ.get("GEMINI_API_KEY"),
            )
        return None

    def load_model(self) -> Any:
        return self._client

    def get_model_name(self) -> str:
        return self.model_id

    def _call(self, prompt: str, schema: Any = None) -> Any:
        if self.provider == "bedrock":
            response = self._client.converse(
                modelId=self.model_id,
                messages=[{"role": "user", "content": [{"text": prompt}]}],
            )
            text = response["output"]["message"]["content"][0]["text"]
        else:
            # openai, openrouter, gemini
            response = self._client.chat.completions.create(
                model=self.model_id,
                messages=[{"role": "user", "content": prompt}],
            )
            text = response.choices[0].message.content or ""

        if schema is not None:
            # deepeval passes a Pydantic model as schema for structured output;
            # extract the JSON block from the response and parse it.
            try:
                start = text.index("{")
                end = text.rindex("}") + 1
                return schema.model_validate(json.loads(text[start:end]))
            except Exception:
                pass

        return text

    def generate(self, prompt: str, schema: Any = None) -> str:
        return self._call(prompt, schema)

    async def a_generate(self, prompt: str, schema: Any = None) -> str:
        import asyncio
        loop = asyncio.get_event_loop()
        return await loop.run_in_executor(None, self._call, prompt, schema)
