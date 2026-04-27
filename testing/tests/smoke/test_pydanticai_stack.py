import pytest

from testing.tests.smoke.harness import SKILLS, assert_high_effort_scores_above_ci_triage, run_skill_case

SDK = "pydanticai"


@pytest.mark.parametrize("skill", SKILLS)
def test_pydanticai_stack_skill(skill: str) -> None:
    run_skill_case(SDK, skill)


def test_pydanticai_stack_adapter_shape() -> None:
    body = run_skill_case(SDK, "architecture")
    event = body["event"]
    assert event["meta"]["extensions"]["chatbot.skill"] == "architecture"
    assert event["meta"]["extensions"]["chatbot.adapter_event_id"] == event["event_id"]
    assert event["task"]["category"] == "planning_architecture"


def test_pydanticai_stack_high_effort_skills_score_above_ci_triage() -> None:
    assert_high_effort_scores_above_ci_triage(SDK)
