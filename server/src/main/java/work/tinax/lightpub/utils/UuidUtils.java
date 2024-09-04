package work.tinax.lightpub.utils;

// from https://gist.github.com/TheNullicorn/d02edf0d67f13df503b4436e6b1e48f1

import java.util.UUID;

import org.eclipse.jdt.annotation.NonNullByDefault;

/**
 * Various utilities for converting to and from "trimmed" UUIDs (UUID strings
 * without hyphens)
 */
@NonNullByDefault
public final class UuidUtils {

    private static final String UUID_REGEX = "[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}";
    private static final String TRIMMED_UUID_REGEX = "[a-f0-9]{12}[a-f0-9]{4}[a-f0-9]{16}";
    private static final String ADD_UUID_HYPHENS_REGEX = "([a-f0-9]{8})([a-f0-9]{4})([a-f0-9]{4})([a-f0-9]{4})([a-f0-9]{12})";

    private UuidUtils() {
    }

    /**
     * @param input A UUID string (may or may not be trimmed)
     * @return The input string as a UUID object
     * @throws IllegalArgumentException If the input string is not a valid
     *                                  trimmed/untrimmed UUID
     */
    public static UUID fromTrimmed(String input) {
        if (!isUuid(input)) {
            throw new IllegalArgumentException("Not a UUID: " + input);

        } else if (input.contains("-")) {
            // Already has hyphens
            return UUID.fromString(input);
        }

        return UUID.fromString(input.replaceAll(ADD_UUID_HYPHENS_REGEX, "$1-$2-$3-$4-$5"));
    }

    /**
     * @return UUID as a string without hyphens
     */
    public static String trim(UUID input) {
        return trim(input.toString());
    }

    /**
     * @return Input string stripped of hyphens
     */
    public static String trim(String input) {
        return input.replace("-", "");
    }

    /**
     * @return Whether or not the input string is a UUID (may or may not be trimmed)
     */
    public static boolean isUuid(String input) {
        return input.matches(TRIMMED_UUID_REGEX) || input.matches(UUID_REGEX);
    }
}