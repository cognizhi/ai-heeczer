package io.github.cognizhi.heeczer;

/**
 * Raised by every {@link HeeczerClient} method on a non-2xx response.
 *
 * <p>The {@code kind} enum mirrors the ingestion service's closed error kind
 * list. Callers should switch on {@link #kind} rather than parsing the message.
 */
public class HeeczerApiException extends RuntimeException {

    private final int status;
    private final ApiErrorKind kind;
    private final String apiMessage;

    public HeeczerApiException(int status, ApiErrorKind kind, String apiMessage) {
        super("heeczer " + status + " " + kind + ": " + apiMessage);
        this.status = status;
        this.kind = kind;
        this.apiMessage = apiMessage;
    }

    /** HTTP status code of the error response. */
    public int getStatus() { return status; }

    /** Structured error kind from the ingestion service envelope. */
    public ApiErrorKind getKind() { return kind; }

    /** Human-readable error message from the ingestion service. */
    public String getApiMessage() { return apiMessage; }
}
