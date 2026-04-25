package io.github.cognizhi.heeczer;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonProperty;

/**
 * Task descriptor block of a canonical event.
 * {@code name} and {@code outcome} are required.
 */
@JsonIgnoreProperties(ignoreUnknown = false)
@JsonInclude(JsonInclude.Include.NON_NULL)
public class EventTask {

    /** Task name (caller-defined, ≤ 256 chars). Required. */
    @JsonProperty("name")
    public String name;

    /**
     * Optional task category slug. Missing or null normalises to
     * {@code "uncategorized"} per PRD §14.2.1.
     */
    @JsonProperty("category")
    public String category;

    @JsonProperty("sub_category")
    public String subCategory;

    /** Task outcome. Required. */
    @JsonProperty("outcome")
    public Outcome outcome;

    public EventTask() {
    }

    public EventTask(String name, Outcome outcome) {
        this.name = name;
        this.outcome = outcome;
    }
}
