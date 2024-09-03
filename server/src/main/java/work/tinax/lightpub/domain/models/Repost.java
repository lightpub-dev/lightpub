package work.tinax.lightpub.domain.models;

import java.time.LocalDateTime;

public final class Repost extends Post {

	private PostId repostOf;

	public Repost(PostId id, URL url, UserId author, PostPrivacy privacy, LocalDateTime createdAt,
			LocalDateTime deletedAt, PostId repostOf) {
		super(id, url, author, privacy, createdAt, deletedAt);
		this.repostOf = repostOf;
	}

	public PostId getRepostOf() {
		return repostOf;
	}

	public void setRepostOf(PostId repostOf) {
		this.repostOf = repostOf;
	}

}
