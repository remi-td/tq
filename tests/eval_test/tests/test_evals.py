"""Pytest test case definitions parameterized from cases JSON files."""

from __future__ import annotations

import pytest
from tests.conftest import load_cases, assert_eval_case


@pytest.mark.parametrize("case", load_cases("base"), ids=lambda c: c["id"])
def test_base(case, bedrock_client, agent_model_id, judge_llm):
    assert_eval_case(case, bedrock_client, agent_model_id, judge_llm)
