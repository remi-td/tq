"""Run single-turn and shallow multi-turn eval cases."""

from __future__ import annotations

import os
from typing import Any

from deepeval.evaluate.configs import CacheConfig, DisplayConfig, ErrorConfig
from deepeval.evaluate.execute import execute_test_cases
from deepeval.test_case import LLMTestCase, ToolCall

from agent.client import run_agent, run_agent_turns
from judge.checks import ToolCallRecord, run_deterministic_checks, get_expected_tools_from_case
from judge.metrics import clarification_metric, get_metrics, tool_correctness_metric
from judge.report import CaseEvalResult, build_recommendation, record_case_result

MAX_TURNS = 7


def _extract_exception_detail(exc: Exception) -> str:
    """Extract detailed error message from exception, unwrapping ExceptionGroup if needed."""
    if hasattr(exc, 'exceptions') and exc.exceptions:
        first_exc = exc.exceptions[0]
        return _extract_exception_detail(first_exc)
    if hasattr(exc, '__cause__') and exc.__cause__:
        return f"{type(exc).__name__}: {str(exc)} (caused by {type(exc.__cause__).__name__}: {str(exc.__cause__)})"
    return f"{type(exc).__name__}: {str(exc)}"


def validate_multi_turn_case(case: dict) -> None:
    """Validate a shallow multi-turn case schema."""
    turns = case.get("turns")
    if turns is None:
        return

    if not isinstance(turns, list):
        raise ValueError(f"[{case.get('id')}] turns must be a list")

    if len(turns) < 2:
        raise ValueError(f"[{case.get('id')}] multi-turn cases need at least 2 turns")

    if len(turns) > MAX_TURNS:
        raise ValueError(f"[{case.get('id')}] multi-turn cases allow at most {MAX_TURNS} turns")

    for index, turn in enumerate(turns, start=1):
        is_clarification = turn.get("expect") == "clarification"
        has_tools = bool(turn.get("expected_tools"))
        if is_clarification == has_tools:
            raise ValueError(
                f"[{case.get('id')}] turn {index} must set exactly one of "
                "'expect': 'clarification' or non-empty 'expected_tools'",
            )
        if "input" not in turn:
            raise ValueError(f"[{case.get('id')}] turn {index} is missing 'input'")


def _to_tool_calls(records: list[ToolCallRecord]) -> list[ToolCall]:
    return [ToolCall(name=tc.name, input_parameters=tc.input_parameters) for tc in records]


def _tool_dicts(records: list[ToolCallRecord]) -> list[dict[str, Any]]:
    return [{"name": tc.name, "params": tc.input_parameters} for tc in records]


def _make_test_case(
    *,
    user_input: str,
    response: str,
    tools_called: list[ToolCallRecord],
    expected_tools_raw: list[dict[str, Any]],
    expected_output_text: str | None = None,
) -> LLMTestCase:
    return LLMTestCase(
        input=user_input,
        actual_output=response,
        expected_output=expected_output_text,
        tools_called=_to_tool_calls(tools_called),
        expected_tools=[
            ToolCall(name=t["name"], input_parameters=t.get("params", {}))
            for t in expected_tools_raw
        ],
    )


def _evaluate_metrics(test_case: LLMTestCase, metrics) -> tuple[list[str], bool]:
    test_result = execute_test_cases(
        [test_case],
        metrics,
        error_config=ErrorConfig(ignore_errors=False, skip_on_missing_params=False),
        display_config=DisplayConfig(verbose_mode=False, show_indicator=False),
        cache_config=CacheConfig(write_cache=False, use_cache=False),
        identifier="eval",
        _use_bar_indicator=False,
        _is_assert_test=True,
    )[0]

    if test_result.success:
        return [], True

    reasons: list[str] = []
    for metric_data in test_result.metrics_data or []:
        if metric_data.error is not None or not metric_data.success:
            detail = metric_data.reason or metric_data.error or "metric failed"
            reasons.append(f"{metric_data.name}: {detail}")
    return reasons, False


def _failure_result(
    case: dict,
    *,
    case_input: str,
    failure_stage: str,
    failure_detail: str,
    expected_tools: list[dict[str, Any]] | None = None,
    actual_tools: list[dict[str, Any]] | None = None,
    actual_output: str | None = None,
    metric_reasons: list[str] | None = None,
    turn_details: list[dict[str, Any]] | None = None,
    input_tokens: int = 0,
    output_tokens: int = 0,
    total_tokens: int = 0,
    cost: float = 0.0,
) -> CaseEvalResult:
    expected = expected_tools if expected_tools is not None else case.get("expected_tools", [])
    metric_reasons = metric_reasons or []
    recommendation = build_recommendation(
        case,
        failure_stage=failure_stage,
        failure_detail=failure_detail,
        expected_tools=expected,
        actual_tools=actual_tools,
        metric_reasons=metric_reasons,
    )
    return CaseEvalResult(
        case_id=case.get("id", "<unknown>"),
        case_type=case.get("type", "happy_path"),
        description=case.get("description", ""),
        input=case_input,
        expected_tools=expected,
        passed=False,
        failure_stage=failure_stage,
        failure_detail=failure_detail,
        actual_tools=actual_tools,
        actual_output=actual_output,
        metric_reasons=metric_reasons,
        recommendation=recommendation,
        turn_details=turn_details,
        input_tokens=input_tokens,
        output_tokens=output_tokens,
        total_tokens=total_tokens,
        cost=cost,
    )


def _success_result(
    case: dict,
    *,
    case_input: str,
    expected_tools: list[dict[str, Any]],
    actual_tools: list[dict[str, Any]],
    actual_output: str,
    turn_details: list[dict[str, Any]] | None = None,
    input_tokens: int = 0,
    output_tokens: int = 0,
    total_tokens: int = 0,
    cost: float = 0.0,
) -> CaseEvalResult:
    return CaseEvalResult(
        case_id=case.get("id", "<unknown>"),
        case_type=case.get("type", "happy_path"),
        description=case.get("description", ""),
        input=case_input,
        expected_tools=expected_tools,
        passed=True,
        actual_tools=actual_tools,
        actual_output=actual_output,
        turn_details=turn_details,
        input_tokens=input_tokens,
        output_tokens=output_tokens,
        total_tokens=total_tokens,
        cost=cost,
    )


def run_single_turn_case(case: dict, bedrock_client, agent_model_id: str, judge_ll) -> CaseEvalResult:
    """Run a single-turn case and return a structured result."""
    validate_multi_turn_case(case)
    if "turns" in case:
        raise ValueError(f"[{case.get('id')}] use run_eval_case() for multi-turn cases")

    # Resolve provider and mode from env
    provider = os.environ.get("EVALS_PROVIDER", "bedrock").lower()
    mode = os.environ.get("EVALS_MODE", "tq-cli")

    try:
        agent_result = run_agent(
            prompt=case["input"],
            provider=provider,
            model_id=agent_model_id,
            mode=mode,
        )
    except Exception as exc:
        return _failure_result(
            case,
            case_input=case["input"],
            failure_stage="agent",
            failure_detail=_extract_exception_detail(exc),
        )

    t_use = getattr(agent_result, "token_usage", None)
    in_t = t_use.input_tokens if t_use else 0
    out_t = t_use.output_tokens if t_use else 0
    tot_t = t_use.total_tokens if t_use else 0
    cost_val = t_use.cost if t_use else 0.0

    raw_calls = [
        ToolCallRecord(name=tc.name, input_parameters=tc.input_parameters)
        for tc in agent_result.tool_calls
    ]
    actual_tools = _tool_dicts(raw_calls)
    expected_tools_raw = get_expected_tools_from_case(case)
    det_errors = run_deterministic_checks(case, raw_calls, agent_result.final_response)
    if det_errors:
        return _failure_result(
            case,
            case_input=case["input"],
            failure_stage="deterministic",
            failure_detail="; ".join(det_errors),
            expected_tools=expected_tools_raw,
            actual_tools=actual_tools,
            actual_output=agent_result.final_response,
            input_tokens=in_t,
            output_tokens=out_t,
            total_tokens=tot_t,
            cost=cost_val,
        )

    # For CLI and no-tool modes, if deterministic check passed (i.e., correct command matched),
    # and case is NOT missing_parameter, return success immediately
    if mode in ("tq-cli", "tq-cli-force-skill", "tq-cli-no-skill", "no-tool") and case.get("type") != "missing_parameter":
        return _success_result(
            case,
            case_input=case["input"],
            expected_tools=expected_tools_raw,
            actual_tools=actual_tools,
            actual_output=agent_result.final_response,
            input_tokens=in_t,
            output_tokens=out_t,
            total_tokens=tot_t,
            cost=cost_val,
        )

    test_case = _make_test_case(
        user_input=case["input"],
        response=agent_result.final_response,
        tools_called=raw_calls,
        expected_tools_raw=expected_tools_raw,
        expected_output_text=case.get("expected_output"),
    )
    metric_reasons, passed = _evaluate_metrics(test_case, get_metrics(case, judge_ll))
    if not passed:
        return _failure_result(
            case,
            case_input=case["input"],
            failure_stage="metric",
            failure_detail="; ".join(metric_reasons),
            actual_tools=actual_tools,
            actual_output=agent_result.final_response,
            metric_reasons=metric_reasons,
            input_tokens=in_t,
            output_tokens=out_t,
            total_tokens=tot_t,
            cost=cost_val,
        )

    return _success_result(
        case,
        case_input=case["input"],
        expected_tools=case.get("expected_tools", []),
        actual_tools=actual_tools,
        actual_output=agent_result.final_response,
        input_tokens=in_t,
        output_tokens=out_t,
        total_tokens=tot_t,
        cost=cost_val,
    )


def run_multi_turn_case(case: dict, bedrock_client, agent_model_id: str, judge_ll) -> CaseEvalResult:
    """Run and score a shallow multi-turn case (2–7 turns)."""
    validate_multi_turn_case(case)
    turns = case["turns"]
    prompts = [turn["input"] for turn in turns]
    case_input = " | ".join(f"Turn {index}: {turn['input']}" for index, turn in enumerate(turns, start=1))

    provider = os.environ.get("EVALS_PROVIDER", "bedrock").lower()
    mode = os.environ.get("EVALS_MODE", "tq-cli")
    max_steps_per_turn = int(os.environ.get("AGENT_MAX_STEPS_PER_TURN", "3"))
    try:
        turn_results = run_agent_turns(
            prompts=prompts,
            provider=provider,
            model_id=agent_model_id,
            mode=mode,
            max_steps_per_turn=max_steps_per_turn,
        )
    except Exception as exc:
        return _failure_result(
            case,
            case_input=case_input,
            failure_stage="agent",
            failure_detail=_extract_exception_detail(exc),
            expected_tools=[],
        )

    in_t = sum(getattr(tr, "token_usage", TokenUsage()).input_tokens for tr in turn_results)
    out_t = sum(getattr(tr, "token_usage", TokenUsage()).output_tokens for tr in turn_results)
    tot_t = sum(getattr(tr, "token_usage", TokenUsage()).total_tokens for tr in turn_results)
    cost_val = sum(getattr(tr, "token_usage", TokenUsage()).cost for tr in turn_results)

    conversation_prefix = ""
    turn_details: list[dict[str, Any]] = []

    for turn_number, (turn_spec, turn_result) in enumerate(zip(turns, turn_results, strict=True), start=1):
        raw_calls = [
            ToolCallRecord(name=tc.name, input_parameters=tc.input_parameters)
            for tc in turn_result.tool_calls
        ]
        actual_tools = _tool_dicts(raw_calls)
        turn_label = f"{case.get('id')} turn {turn_number}"
        turn_input = f"{conversation_prefix}User: {turn_spec['input']}"

        if turn_spec.get("expect") == "clarification":
            pseudo_case = {"id": turn_label, "type": "missing_parameter", "expected_tools": []}
            det_errors = run_deterministic_checks(pseudo_case, raw_calls, turn_result.final_response)
            if det_errors:
                turn_details.append(
                    {
                        "turn": turn_number,
                        "input": turn_spec["input"],
                        "mode": "clarification",
                        "passed": False,
                        "failure_stage": "deterministic",
                        "failure_detail": "; ".join(det_errors),
                        "actual_tools": actual_tools,
                    }
                )
                return _failure_result(
                    case,
                    case_input=case_input,
                    failure_stage="deterministic",
                    failure_detail=f"turn {turn_number}: {'; '.join(det_errors)}",
                    expected_tools=[],
                    actual_tools=actual_tools,
                    actual_output=turn_result.final_response,
                    turn_details=turn_details,
                    input_tokens=in_t,
                    output_tokens=out_t,
                    total_tokens=tot_t,
                    cost=cost_val,
                )

            test_case = _make_test_case(
                user_input=turn_input,
                response=turn_result.final_response,
                tools_called=[],
                expected_tools_raw=[],
                expected_output_text=turn_spec.get("expected_output"),
            )
            metric_reasons, passed = _evaluate_metrics(test_case, [clarification_metric(judge_ll)])
        else:
            expected_tools = get_expected_tools_from_case(turn_spec)
            pseudo_case = {
                "id": turn_label,
                "type": "happy_path",
                "expected_tools": expected_tools,
            }
            det_errors = run_deterministic_checks(pseudo_case, raw_calls, turn_result.final_response)
            if det_errors:
                turn_details.append(
                    {
                        "turn": turn_number,
                        "input": turn_spec["input"],
                        "mode": "tool",
                        "passed": False,
                        "failure_stage": "deterministic",
                        "failure_detail": "; ".join(det_errors),
                        "expected_tools": expected_tools,
                        "actual_tools": actual_tools,
                    }
                )
                return _failure_result(
                    case,
                    case_input=case_input,
                    failure_stage="deterministic",
                    failure_detail=f"turn {turn_number}: {'; '.join(det_errors)}",
                    expected_tools=expected_tools,
                    actual_tools=actual_tools,
                    actual_output=turn_result.final_response,
                    turn_details=turn_details,
                    input_tokens=in_t,
                    output_tokens=out_t,
                    total_tokens=tot_t,
                    cost=cost_val,
                )

            # For CLI and no-tool modes, bypass metrics evaluation for happy path turns
            if mode in ("tq-cli", "tq-cli-force-skill", "tq-cli-no-skill", "no-tool"):
                passed = True
                metric_reasons = []
            else:
                test_case = _make_test_case(
                    user_input=turn_input,
                    response=turn_result.final_response,
                    tools_called=raw_calls,
                    expected_tools_raw=expected_tools,
                    expected_output_text=turn_spec.get("expected_output"),
                )
                metric_reasons, passed = _evaluate_metrics(test_case, [tool_correctness_metric(judge_ll)])

        if not passed:
            turn_details.append(
                {
                    "turn": turn_number,
                    "input": turn_spec["input"],
                    "mode": "clarification" if turn_spec.get("expect") == "clarification" else "tool",
                    "passed": False,
                    "failure_stage": "metric",
                    "failure_detail": "; ".join(metric_reasons),
                    "expected_tools": turn_spec.get("expected_tools", []),
                    "actual_tools": actual_tools,
                }
            )
            return _failure_result(
                case,
                case_input=case_input,
                failure_stage="metric",
                failure_detail=f"turn {turn_number}: {'; '.join(metric_reasons)}",
                expected_tools=turn_spec.get("expected_tools", []),
                actual_tools=actual_tools,
                actual_output=turn_result.final_response,
                metric_reasons=metric_reasons,
                turn_details=turn_details,
                input_tokens=in_t,
                output_tokens=out_t,
                total_tokens=tot_t,
                cost=cost_val,
            )

        turn_details.append(
            {
                "turn": turn_number,
                "input": turn_spec["input"],
                "mode": "clarification" if turn_spec.get("expect") == "clarification" else "tool",
                "passed": True,
                "actual_tools": actual_tools,
            }
        )
        conversation_prefix += f"User: {turn_spec['input']}\nAssistant: {turn_result.final_response}\n"

    last_turn = turn_results[-1]
    last_tools = [
        ToolCallRecord(name=tc.name, input_parameters=tc.input_parameters)
        for tc in last_turn.tool_calls
    ]
    return _success_result(
        case,
        case_input=case_input,
        expected_tools=turns[-1].get("expected_tools", []),
        actual_tools=_tool_dicts(last_tools),
        actual_output=last_turn.final_response,
        turn_details=turn_details,
        input_tokens=in_t,
        output_tokens=out_t,
        total_tokens=tot_t,
        cost=cost_val,
    )


def run_eval_case(case: dict, bedrock_client, agent_model_id: str, judge_ll) -> CaseEvalResult:
    """Run any eval case and return a structured result."""
    validate_multi_turn_case(case)
    if "turns" in case:
        return run_multi_turn_case(case, bedrock_client, agent_model_id, judge_ll)
    return run_single_turn_case(case, bedrock_client, agent_model_id, judge_ll)


def assert_eval_case(case: dict, bedrock_client, agent_model_id: str, judge_ll) -> None:
    """Run and score any eval case (single- or multi-turn)."""
    result = run_eval_case(case, bedrock_client, agent_model_id, judge_ll)
    record_case_result(result)
    if not result.passed:
        detail = result.failure_detail or "; ".join(result.metric_reasons) or "eval case failed"
        raise AssertionError(f"[{result.case_id}] {result.failure_stage} check failed: {detail}")
