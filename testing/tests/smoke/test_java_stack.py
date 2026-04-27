import pytest

from testing.tests.smoke.harness import SKILLS, assert_high_effort_scores_above_ci_triage, run_skill_case

SDK = "java"


@pytest.mark.parametrize("skill", SKILLS)
def test_java_stack_skill(skill: str) -> None:
    run_skill_case(SDK, skill)


def test_java_stack_high_effort_skills_score_above_ci_triage() -> None:
    assert_high_effort_scores_above_ci_triage(SDK)
