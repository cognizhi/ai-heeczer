package io.github.cognizhi.heeczer;

import com.fasterxml.jackson.annotation.JsonProperty;

/** Returned by {@link HeeczerClient#version}. */
public final class VersionResponse {
    @JsonProperty("ok")               public boolean ok;
    @JsonProperty("envelope_version") public String envelopeVersion;
    @JsonProperty("scoring_version")  public String scoringVersion;
    @JsonProperty("spec_version")     public String specVersion;
    @JsonProperty("service")          public String service;
}
