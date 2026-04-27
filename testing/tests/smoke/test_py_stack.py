import pytest

from testing.tests.smoke.harness import SKILLS, assert_high_effort_scores_above_ci_triage, run_skill_case

SDK = "py"


@pytest.mark.parametrize("skill", SKILLS)
def test_py_stack_skill(skill: str) -> None:
    run_skill_case(SDK, skill)


def test_py_stack_high_effort_skills_score_above_ci_triage() -> None:
    assert_high_effort_scores_above_ci_triage(SDK)
