package work.tinax.lightpub.domain.models;

import java.time.LocalDateTime;
import java.util.List;
import java.util.Optional;

public final class NormalPost extends Post {

	private PostContent content;
	private List<UserId> mentionedUsers;
	private PostCounter counter;
	private Optional<PostId> replyTo;
	private Optional<PostId> quoteOf;

	public NormalPost(PostId id, Optional<URL> url, UserId author, PostPrivacy privacy, LocalDateTime createdAt,
			LocalDateTime deletedAt, PostContent content, List<UserId> mentionedUsers, PostCounter counter,
			Optional<PostId> replyTo, Optional<PostId> quoteOf) {
		super(id, url, author, privacy, createdAt, deletedAt);

		if (replyTo != null && quoteOf != null) {
			throw new IllegalArgumentException("both replyTo and quoteOf cannot be set");
		}

		this.content = content;
		this.mentionedUsers = mentionedUsers;
		this.counter = counter;
		this.replyTo = replyTo;
		this.quoteOf = quoteOf;
	}

	public Optional<PostId> getQuoteOf() {
		return quoteOf;
	}

	public void setQuoteOf(Optional<PostId> quoteOf) {
		this.quoteOf = quoteOf;
	}

	public Optional<PostId> getReplyTo() {
		return replyTo;
	}

	public void setReplyTo(Optional<PostId> replyTo) {
		this.replyTo = replyTo;
	}

	public PostCounter getCounter() {
		return counter;
	}

	public void setCounter(PostCounter counter) {
		this.counter = counter;
	}

	public List<UserId> getMentionedUsers() {
		return mentionedUsers;
	}

	public void setMentionedUsers(List<UserId> mentionedUsers) {
		this.mentionedUsers = mentionedUsers;
	}

	public PostContent getContent() {
		return content;
	}

	public void setContent(PostContent content) {
		this.content = content;
	}

}
