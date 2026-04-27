package com.cognizhi.heeczer.teststack.tools;

import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

public final class Catalogue {
    private Catalogue() {
    }

    public record ToolTraceEntry(String tool_name, int invoked_at_ms, double output_size) {
    }

    private static final Map<String, Double> OUTPUT_SIZES = Map.of(
            "web_search", 0.0,
            "code_executor", 0.5,
            "document_reader", 0.0,
            "data_analyst", 1.0,
            "plan_reviewer", 0.0,
            "risk_checker", 0.0,
            "summarizer", 0.8,
            "diff_generator", 0.3);

    public static List<ToolTraceEntry> traceForTools(List<String> activeTools) {
        List<ToolTraceEntry> trace = new ArrayList<>();
        for (int index = 0; index < activeTools.size(); index++) {
            String toolName = activeTools.get(index);
            trace.add(new ToolTraceEntry(toolName, index * 25, OUTPUT_SIZES.getOrDefault(toolName, 0.0)));
        }
        return trace;
    }

    public static List<Map<String, Object>> functionSchemas(List<String> activeTools) {
        List<Map<String, Object>> schemas = new ArrayList<>();
        for (String toolName : activeTools) {
            Map<String, Object> input = new LinkedHashMap<>();
            input.put("type", "string");
            Map<String, Object> properties = new LinkedHashMap<>();
            properties.put("input", input);
            Map<String, Object> parameters = new LinkedHashMap<>();
            parameters.put("type", "object");
            parameters.put("properties", properties);
            parameters.put("required", List.of("input"));
            Map<String, Object> function = new LinkedHashMap<>();
            function.put("name", toolName);
            function.put("description", "Synthetic " + toolName + " tool used by the ai-heeczer local stack.");
            function.put("parameters", parameters);
            Map<String, Object> schema = new LinkedHashMap<>();
            schema.put("type", "function");
            schema.put("function", function);
            schemas.add(schema);
        }
        return schemas;
    }
}
