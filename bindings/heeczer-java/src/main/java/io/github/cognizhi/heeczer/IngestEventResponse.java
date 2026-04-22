package io.github.cognizhi.heeczer;

import com.fasterxml.jackson.annotation.JsonProperty;

/** Returned by {@link HeeczerClient#ingestEvent}. */
public final class IngestEventResponse {
    @JsonProperty("ok")               public boolean ok;
    @JsonProperty("envelope_version") public String envelopeVersion;
    @JsonProperty("event_id")         public String eventId;
    @JsonProperty("score")            public ScoreResult score;
}
