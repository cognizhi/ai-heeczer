package io.github.cognizhi.heeczer;

import com.fasterxml.jackson.annotation.JsonProperty;

/** Returned by {@link HeeczerClient#testScorePipeline}. */
public final class TestPipelineResponse {
    @JsonProperty("ok")               public boolean ok;
    @JsonProperty("envelope_version") public String envelopeVersion;
    @JsonProperty("score")            public ScoreResult score;
}
