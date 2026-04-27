package com.cognizhi.heeczer.teststack.skills;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;

public final class SkillCatalogue {
    private static final ObjectMapper MAPPER = new ObjectMapper();
    private static final Map<String, String> ALIASES = Map.of(
            "code-gen", "code_gen",
            "doc-summary", "doc_summary",
            "ci-triage", "ci_triage");

    private SkillCatalogue() {
    }

    public static String normalize(String rawSkill) {
        String selected = rawSkill == null || rawSkill.isBlank() ? "code_gen"
                : rawSkill.replaceFirst("^/skill\\s+", "").trim();
        return ALIASES.getOrDefault(selected, selected);
    }

    public static JsonNode load(String rawSkill) throws IOException {
        String fixtureRoot = System.getenv().getOrDefault("SKILL_FIXTURE_DIR", "/fixtures/skills");
        Path fixturePath = Path.of(fixtureRoot, normalize(rawSkill) + ".json");
        return MAPPER.readTree(Files.readString(fixturePath));
    }

    public static List<String> activeTools(JsonNode fixture) {
        List<String> activeTools = new ArrayList<>();
        for (JsonNode step : fixture.path("mock_script")) {
            activeTools.add(step.path("tool").asText());
        }
        return activeTools;
    }
}
