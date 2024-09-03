package work.tinax.lightpub.domain.models;

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
}
