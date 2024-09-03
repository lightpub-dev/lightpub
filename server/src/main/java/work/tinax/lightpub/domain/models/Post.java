package work.tinax.lightpub.domain.models;

import java.time.LocalDateTime;
import java.util.Optional;

import org.eclipse.jdt.annotation.Nullable;

public sealed class Post permits NormalPost, Repost {
	private PostId id;
	private Optional<URL> url;
	private UserId author;
	private PostPrivacy privacy;
	private LocalDateTime createdAt;
	private LocalDateTime deletedAt;

	public Post(PostId id, Optional<URL> url, UserId author, PostPrivacy privacy, LocalDateTime createdAt,
			LocalDateTime deletedAt) {
		this.id = id;
		this.url = url;
		this.author = author;
		this.privacy = privacy;
		this.createdAt = createdAt;
		this.deletedAt = deletedAt;
	}

	public LocalDateTime getDeletedAt() {
		return deletedAt;
	}

	public void setDeletedAt(LocalDateTime deletedAt) {
		this.deletedAt = deletedAt;
	}

	public LocalDateTime getCreatedAt() {
		return createdAt;
	}

	public void setCreatedAt(LocalDateTime createdAt) {
		this.createdAt = createdAt;
	}

	public PostPrivacy getPrivacy() {
		return privacy;
	}

	public void setPrivacy(PostPrivacy privacy) {
		this.privacy = privacy;
	}

	public UserId getAuthor() {
		return author;
	}

	public void setAuthor(UserId author) {
		this.author = author;
	}

	@Nullable
	public Optional<URL> getUrl() {
		return url;
	}

	public void setUrl(Optional<URL> url) {
		this.url = url;
	}

	public PostId getId() {
		return id;
	}

	public void setId(PostId id) {
		this.id = id;
	}
}
