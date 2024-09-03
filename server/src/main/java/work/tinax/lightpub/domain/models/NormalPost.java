package work.tinax.lightpub.domain.models;

import java.time.LocalDateTime;
import java.util.List;

public final class NormalPost extends Post {

	private PostContent content;
	private List<UserId> mentionedUsers;
	private PostCounter counter;
	private PostId replyTo;
	private PostId quoteOf;

	public NormalPost(PostId id, URL url, UserId author, PostPrivacy privacy, LocalDateTime createdAt,
			LocalDateTime deletedAt, PostContent content, List<UserId> mentionedUsers, PostCounter counter,
			PostId replyTo, PostId quoteOf) {
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

	public PostId getQuoteOf() {
		return quoteOf;
	}

	public void setQuoteOf(PostId quoteOf) {
		this.quoteOf = quoteOf;
	}

	public PostId getReplyTo() {
		return replyTo;
	}

	public void setReplyTo(PostId replyTo) {
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
