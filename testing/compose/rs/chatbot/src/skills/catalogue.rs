use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct SkillFixture {
    pub skill: String,
    pub mock_script: Vec<MockScriptStep>,
    pub expected_event: ExpectedEvent,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MockScriptStep {
    pub tool: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExpectedEvent {
    pub task: serde_json::Value,
    pub metrics: serde_json::Value,
    pub context: serde_json::Value,
}

pub fn normalize_skill(raw_skill: Option<&str>) -> String {
    let selected = raw_skill
        .unwrap_or("code_gen")
        .trim()
        .strip_prefix("/skill ")
        .unwrap_or_else(|| raw_skill.unwrap_or("code_gen").trim())
        .to_owned();
    match selected.as_str() {
        "code-gen" => "code_gen".to_owned(),
        "doc-summary" => "doc_summary".to_owned(),
        "ci-triage" => "ci_triage".to_owned(),
        "" => "code_gen".to_owned(),
        _ => selected,
    }
}

pub fn load_skill(raw_skill: Option<&str>) -> anyhow::Result<SkillFixture> {
    let fixture_root =
        std::env::var("SKILL_FIXTURE_DIR").unwrap_or_else(|_| "/fixtures/skills".to_owned());
    let mut fixture_path = PathBuf::from(fixture_root);
    fixture_path.push(format!("{}.json", normalize_skill(raw_skill)));
    let body = std::fs::read_to_string(&fixture_path)?;
    Ok(serde_json::from_str(&body)?)
}

pub fn active_tools(fixture: &SkillFixture) -> Vec<String> {
    fixture
        .mock_script
        .iter()
        .map(|step| step.tool.clone())
        .collect()
}
