package io.github.cognizhi.heeczer;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonProperty;

/**
 * Optional identity block of a canonical event.
 * All fields are optional; null values are omitted from serialized JSON.
 */
@JsonIgnoreProperties(ignoreUnknown = false)
@JsonInclude(JsonInclude.Include.NON_NULL)
public class EventIdentity {

    @JsonProperty("user_id")
    public String userId;

    @JsonProperty("team_id")
    public String teamId;

    @JsonProperty("business_unit_id")
    public String businessUnitId;

    /** Resolved against the active TierSet (PRD §14.2.1). */
    @JsonProperty("tier_id")
    public String tierId;

    public EventIdentity() {
    }
}
