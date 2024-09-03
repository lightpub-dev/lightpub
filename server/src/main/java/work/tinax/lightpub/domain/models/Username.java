package work.tinax.lightpub.domain.models;

import org.eclipse.jdt.annotation.Nullable;

public class Username {
	private String username;

	public String getUsername() {
		return username;
	}

	public Username(String username) {
		// validation
		int len = username.length();
		if (len < 3 || 16 < len) {
			throw new IllegalArgumentException("username must be 3-16 length");
		}

		boolean hasIllegalChar = false;
		for (int i = 0; i < len; i++) {
			char ch = username.charAt(i);

			if (i == 0) {
				if (ch == '-' || ch == '_') {
					hasIllegalChar = true;
					break;
				}
			}

			if (ch == '-' || ch == '_' || Character.isAlphabetic(ch) || Character.isDigit(ch)) {
				continue;
			}
		}

		if (username.contains("--") || username.contains("__") || username.contains("-_") || username.contains("_-")) {
			hasIllegalChar = true;
		}

		if (hasIllegalChar) {
			throw new IllegalArgumentException("username contains prohibited char");
		}

		this.username = username;
	}

	@Override
	public boolean equals(@Nullable Object o) {
		if (o instanceof Username u) {
			return username.equals(u.username);
		}
		return false;
	}

	@Override
	public int hashCode() {
		return username.hashCode();
	}
}
