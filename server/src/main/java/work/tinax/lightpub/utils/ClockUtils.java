package work.tinax.lightpub.utils;

import java.time.LocalDateTime;
import java.time.ZoneId;
import java.util.Objects;

import org.eclipse.jdt.annotation.NonNullByDefault;

@NonNullByDefault
public class ClockUtils {
	public static LocalDateTime now() {
		return Objects.requireNonNull(LocalDateTime.now(ZoneId.of("UTC")));
	}
}
