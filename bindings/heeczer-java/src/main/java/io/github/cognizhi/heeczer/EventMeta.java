package io.github.cognizhi.heeczer;

import com.fasterxml.jackson.annotation.JsonAnyGetter;
import com.fasterxml.jackson.annotation.JsonAnySetter;
import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.databind.JsonNode;

/**
 * SDK metadata block of a canonical event.
 * {@code sdk_language} and {@code sdk_version} are required.
 * {@code extensions} is the sole permitted bucket for unknown fields (PRD §13 /
 * ADR-0002).
 */
@JsonIgnoreProperties(ignoreUnknown = false)
@JsonInclude(JsonInclude.Include.NON_NULL)
public class EventMeta {

    /**
     * SDK language identifier ({@code "java"}, {@code "node"}, {@code "python"},
     * …). Required.
     */
    @JsonProperty("sdk_language")
    public String sdkLanguage;

    /** SDK semver string. Required. */
    @JsonProperty("sdk_version")
    public String sdkVersion;

    /** Override scoring profile id. Null uses the workspace default. */
    @JsonProperty("scoring_profile")
    public String scoringProfile;

    /**
     * Sole permitted location for custom / unknown fields (PRD §13).
     * Value is any JSON object.
     */
    @JsonProperty("extensions")
    public JsonNode extensions;

    public EventMeta() {
    }

    public EventMeta(String sdkLanguage, String sdkVersion) {
        this.sdkLanguage = sdkLanguage;
        this.sdkVersion = sdkVersion;
    }
}
