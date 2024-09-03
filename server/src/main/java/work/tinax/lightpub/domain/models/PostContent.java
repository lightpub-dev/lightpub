package work.tinax.lightpub.domain.models;

import org.eclipse.jdt.annotation.Nullable;

public class PostContent {
	private String content;

	public PostContent(String content) {
		this.content = content;
	}

	public String getContent() {
		return content;
	}

	@Override
	public boolean equals(@Nullable Object o) {
		if (o instanceof PostContent p) {
			return content.equals(p.content);
		}
		return false;
	}

	@Override
	public int hashCode() {
		return content.hashCode();
	}
}
