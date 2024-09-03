package work.tinax.lightpub.domain.models;

import org.eclipse.jdt.annotation.Nullable;

public class URL {
	private String url;

	public URL(String url) {
		if (url.length() > 512) {
			throw new IllegalArgumentException("url is too long");
		}

		if (!url.startsWith("http://") && !url.startsWith("https://")) {
			throw new IllegalArgumentException("url is neither http:// nor https://");
		}

		this.url = url;
	}

	public String getUrl() {
		return url;
	}

	@Override
	public boolean equals(@Nullable Object o) {
		if (o instanceof URL u) {
			return url.equals(u.url);
		}
		return false;
	}

	@Override
	public int hashCode() {
		return url.hashCode();
	}
}
