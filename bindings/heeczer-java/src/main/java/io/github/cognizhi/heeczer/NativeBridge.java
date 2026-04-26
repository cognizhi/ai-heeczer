package io.github.cognizhi.heeczer;

import com.fasterxml.jackson.databind.DeserializationFeature;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import java.io.File;
import java.io.IOException;
import java.lang.invoke.MethodHandle;
import java.lang.reflect.Array;
import java.lang.reflect.Method;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Locale;
import java.util.Optional;

interface NativeBridge {
    NativeVersion version();

    ScoreResult score(Object event, Object profile, Object tiers, String tierOverride)
            throws IOException;

    static NativeBridge create() {
        return NativeFfmBridge.create();
    }
}

record NativeVersion(String scoringVersion, String specVersion) {
}

final class NativeFfmBridge implements NativeBridge {
    private static final ObjectMapper MAPPER = new ObjectMapper()
            .configure(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES, false);
    private static final String LIBRARY_PROPERTY = "heeczer.native.library";
    private static final String LIBRARY_ENV = "HEECZER_NATIVE_LIBRARY";
    private static final String DEFAULT_LIBRARY = "heeczer_core_c";

    private final MethodHandle scoreJson;
    private final MethodHandle versionsJson;
    private final MethodHandle freeString;
    private final Method ofConfinedArena;
    private final Method allocateUtf8String;
    private final Method reinterpret;
    private final Method getUtf8String;
    private final Object nullSegment;

    private NativeFfmBridge() throws ReflectiveOperationException {
        Class<?> arenaClass = Class.forName("java.lang.foreign.Arena");
        Class<?> linkerClass = Class.forName("java.lang.foreign.Linker");
        Class<?> linkerOptionClass = Class.forName("java.lang.foreign.Linker$Option");
        Class<?> symbolLookupClass = Class.forName("java.lang.foreign.SymbolLookup");
        Class<?> segmentAllocatorClass = Class.forName("java.lang.foreign.SegmentAllocator");
        Class<?> memorySegmentClass = Class.forName("java.lang.foreign.MemorySegment");
        Class<?> functionDescriptorClass = Class.forName("java.lang.foreign.FunctionDescriptor");
        Class<?> memoryLayoutClass = Class.forName("java.lang.foreign.MemoryLayout");
        Class<?> memoryLayoutArrayClass = Array.newInstance(memoryLayoutClass, 0).getClass();
        Class<?> linkerOptionArrayClass = Array.newInstance(linkerOptionClass, 0).getClass();

        Object globalArena = arenaClass.getMethod("global").invoke(null);
        Object linker = linkerClass.getMethod("nativeLinker").invoke(null);
        Object lookup = resolveLookup(symbolLookupClass, arenaClass, globalArena);
        Object addressLayout = Class.forName("java.lang.foreign.ValueLayout")
                .getField("ADDRESS")
                .get(null);

        Method ofDescriptor = functionDescriptorClass.getMethod(
                "of",
                memoryLayoutClass,
                memoryLayoutArrayClass);
        Method ofVoidDescriptor = functionDescriptorClass.getMethod(
                "ofVoid",
                memoryLayoutArrayClass);
        Method downcallHandle = linkerClass.getMethod(
                "downcallHandle",
                memorySegmentClass,
                functionDescriptorClass,
                linkerOptionArrayClass);

        Object scoreDescriptor = ofDescriptor.invoke(
                null,
                addressLayout,
                arrayOf(memoryLayoutClass, addressLayout, addressLayout, addressLayout, addressLayout));
        Object versionDescriptor = ofDescriptor.invoke(
                null,
                addressLayout,
                arrayOf(memoryLayoutClass));
        Object freeDescriptor = ofVoidDescriptor.invoke(
                null,
                arrayOf(memoryLayoutClass, addressLayout));

        this.scoreJson = (MethodHandle) downcallHandle.invoke(
                linker,
                requireSymbol(symbolLookupClass, lookup, "heeczer_score_json"),
                scoreDescriptor,
                Array.newInstance(linkerOptionClass, 0));
        this.versionsJson = (MethodHandle) downcallHandle.invoke(
                linker,
                requireSymbol(symbolLookupClass, lookup, "heeczer_versions_json"),
                versionDescriptor,
                Array.newInstance(linkerOptionClass, 0));
        this.freeString = (MethodHandle) downcallHandle.invoke(
                linker,
                requireSymbol(symbolLookupClass, lookup, "heeczer_free_string"),
                freeDescriptor,
                Array.newInstance(linkerOptionClass, 0));
        this.ofConfinedArena = arenaClass.getMethod("ofConfined");
        this.allocateUtf8String = segmentAllocatorClass.getMethod("allocateUtf8String", String.class);
        this.reinterpret = memorySegmentClass.getMethod("reinterpret", long.class);
        this.getUtf8String = memorySegmentClass.getMethod("getUtf8String", long.class);
        this.nullSegment = memorySegmentClass.getField("NULL").get(null);
    }

    static NativeBridge create() {
        int runtimeFeature = Runtime.version().feature();
        if (runtimeFeature < 22) {
            throw new IllegalStateException(
                    "native mode requires JDK 22+ FFM runtime; current JVM is " + runtimeFeature);
        }
        try {
            return new NativeFfmBridge();
        } catch (ReflectiveOperationException err) {
            throw new IllegalStateException(
                    "native mode could not initialize java.lang.foreign; ensure a JDK 22+ runtime is in use",
                    err);
        }
    }

    @Override
    public NativeVersion version() {
        String body = invokeJson(versionsJson);
        try {
            JsonNode root = MAPPER.readTree(body);
            return new NativeVersion(
                    root.path("scoring_version").asText(),
                    root.path("spec_version").asText());
        } catch (IOException err) {
            throw new IllegalStateException("native mode returned invalid version JSON", err);
        }
    }

    @Override
    public ScoreResult score(Object event, Object profile, Object tiers, String tierOverride)
            throws IOException {
        String body = invokeScore(
                writeJson(event),
                writeJsonOrNull(profile),
                writeJsonOrNull(tiers),
                tierOverride);
        JsonNode root = MAPPER.readTree(body);
        if (!root.path("ok").asBoolean(false)) {
            throw toNativeException(root);
        }
        JsonNode result = root.get("result");
        if (result == null || result.isNull()) {
            throw new HeeczerApiException(500, ApiErrorKind.unknown,
                    "native mode returned an empty result envelope");
        }
        return MAPPER.treeToValue(result, ScoreResult.class);
    }

    private static Object requireSymbol(Class<?> symbolLookupClass, Object lookup, String name)
            throws ReflectiveOperationException {
        Method find = symbolLookupClass.getMethod("find", String.class);
        Optional<?> symbol = (Optional<?>) find.invoke(lookup, name);
        return symbol.orElseThrow(() -> new IllegalStateException(
                "native mode could not resolve symbol '" + name + "' in heeczer_core_c"));
    }

    private static Object resolveLookup(Class<?> symbolLookupClass, Class<?> arenaClass, Object globalArena)
            throws ReflectiveOperationException {
        String configured = configuredLibrary();
        if (configured == null) {
            return loadLibraryByName(symbolLookupClass, DEFAULT_LIBRARY);
        }

        if (looksLikePath(configured)) {
            Path libraryPath = Path.of(configured).toAbsolutePath();
            if (!Files.exists(libraryPath)) {
                throw new IllegalStateException(
                        "native mode library path does not exist: " + libraryPath);
            }
            return symbolLookupClass
                    .getMethod("libraryLookup", Path.class, arenaClass)
                    .invoke(null, libraryPath, globalArena);
        }

        return loadLibraryByName(symbolLookupClass, configured);
    }

    private static Object loadLibraryByName(Class<?> symbolLookupClass, String libraryName)
            throws ReflectiveOperationException {
        try {
            System.loadLibrary(libraryName);
        } catch (UnsatisfiedLinkError err) {
            throw new IllegalStateException(
                    "native mode could not load '" + libraryName + "'; set -D" + LIBRARY_PROPERTY
                            + "=/absolute/path/to/libheeczer_core_c.so or configure java.library.path",
                    err);
        }
        return symbolLookupClass.getMethod("loaderLookup").invoke(null);
    }

    private String invokeScore(String eventJson, String profileJson, String tiersJson, String tierOverride) {
        Object arena = newArena();
        try {
            Object event = allocateUtf8String.invoke(arena, eventJson);
            Object profile = profileJson == null ? nullSegment : allocateUtf8String.invoke(arena, profileJson);
            Object tiers = tiersJson == null ? nullSegment : allocateUtf8String.invoke(arena, tiersJson);
            Object override = tierOverride == null ? nullSegment : allocateUtf8String.invoke(arena, tierOverride);
            Object raw = scoreJson.invokeWithArguments(event, profile, tiers, override);
            return readOwnedString(raw);
        } catch (Throwable err) {
            throw propagateNativeFailure("score", err);
        } finally {
            closeArena(arena);
        }
    }

    private String invokeJson(MethodHandle handle) {
        try {
            Object raw = handle.invokeWithArguments();
            return readOwnedString(raw);
        } catch (Throwable err) {
            throw propagateNativeFailure("version", err);
        }
    }

    private Object newArena() {
        try {
            return ofConfinedArena.invoke(null);
        } catch (ReflectiveOperationException err) {
            throw new IllegalStateException("native mode could not allocate an FFM arena", err);
        }
    }

    private void closeArena(Object arena) {
        if (arena instanceof AutoCloseable closeable) {
            try {
                closeable.close();
            } catch (Exception err) {
                throw new IllegalStateException("native mode could not close an FFM arena", err);
            }
        }
    }

    private String readOwnedString(Object raw) {
        if (raw == null || raw.equals(nullSegment)) {
            throw new IllegalStateException("native mode returned a null string pointer");
        }
        try {
            Object view = reinterpret.invoke(raw, Long.MAX_VALUE);
            return (String) getUtf8String.invoke(view, 0L);
        } catch (ReflectiveOperationException err) {
            throw new IllegalStateException("native mode could not decode a UTF-8 result", err);
        } finally {
            try {
                freeString.invokeWithArguments(raw);
            } catch (Throwable err) {
                throw propagateNativeFailure("free", err);
            }
        }
    }

    private HeeczerApiException toNativeException(JsonNode root) {
        JsonNode error = root.get("error");
        if (error == null || error.isNull()) {
            return new HeeczerApiException(500, ApiErrorKind.unknown,
                    "native mode returned an error envelope without details");
        }

        String message;
        ApiErrorKind kind;
        if (error.isTextual()) {
            message = error.asText();
            kind = classifyLegacyError(message);
        } else {
            String nativeKind = error.path("kind").asText(null);
            message = error.path("message").asText("native mode returned an error");
            kind = mapNativeKind(nativeKind, message);
        }
        int status = (kind == ApiErrorKind.schema || kind == ApiErrorKind.bad_request) ? 400 : 500;
        return new HeeczerApiException(status, kind, message);
    }

    private static ApiErrorKind mapNativeKind(String nativeKind, String message) {
        if (nativeKind == null || nativeKind.isBlank()) {
            return classifyLegacyError(message);
        }
        return switch (nativeKind) {
            case "schema" -> ApiErrorKind.schema;
            case "deserialise", "nul-input", "invalid-utf8" -> ApiErrorKind.bad_request;
            case "score" -> ApiErrorKind.scoring;
            case "panic" -> ApiErrorKind.unavailable;
            default -> ApiErrorKind.unknown;
        };
    }

    static ApiErrorKind classifyLegacyError(String message) {
        String lowered = message == null ? "" : message.toLowerCase(Locale.ROOT);
        if (lowered.contains("panic")) {
            return ApiErrorKind.unavailable;
        }
        if (lowered.contains("schema") || lowered.contains("validation")) {
            return ApiErrorKind.schema;
        }
        if (lowered.contains("utf-8") || lowered.contains("must be non-null")
                || lowered.contains("expected") || lowered.contains("missing field")) {
            return ApiErrorKind.bad_request;
        }
        if (lowered.contains("tier") || lowered.contains("score")) {
            return ApiErrorKind.scoring;
        }
        return ApiErrorKind.unknown;
    }

    private static boolean looksLikePath(String configured) {
        if (configured == null || configured.isBlank()) {
            return false;
        }
        return configured.indexOf(File.separatorChar) >= 0
                || configured.contains("/")
                || configured.contains("\\")
                || configured.endsWith(".so")
                || configured.endsWith(".dylib")
                || configured.endsWith(".dll")
                || Files.exists(Path.of(configured));
    }

    private static String configuredLibrary() {
        String configured = System.getProperty(LIBRARY_PROPERTY);
        if (configured != null && !configured.isBlank()) {
            return configured;
        }
        configured = System.getenv(LIBRARY_ENV);
        if (configured != null && !configured.isBlank()) {
            return configured;
        }
        return null;
    }

    private static Object arrayOf(Class<?> elementClass, Object... values) {
        Object array = Array.newInstance(elementClass, values.length);
        for (int i = 0; i < values.length; i++) {
            Array.set(array, i, values[i]);
        }
        return array;
    }

    private static RuntimeException propagateNativeFailure(String phase, Throwable err) {
        if (err instanceof RuntimeException runtimeException) {
            return runtimeException;
        }
        return new IllegalStateException("native mode failed during " + phase + " bridge call", err);
    }

    private static String writeJson(Object value) throws IOException {
        return MAPPER.writeValueAsString(value);
    }

    private static String writeJsonOrNull(Object value) throws IOException {
        if (value == null) {
            return null;
        }
        return writeJson(value);
    }
}
