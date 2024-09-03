package work.tinax.lightpub.db.models;

import java.time.LocalDateTime;

import jakarta.persistence.Entity;
import jakarta.persistence.EnumType;
import jakarta.persistence.Enumerated;
import jakarta.persistence.FetchType;
import jakarta.persistence.Id;
import jakarta.persistence.ManyToOne;
import jakarta.persistence.Table;
import jakarta.validation.constraints.NotNull;

@Entity
@Table(name = "post")
public class DBPost {
	@Id
	@NotNull
	private String postId;

	private String url;

	@ManyToOne(fetch = FetchType.LAZY)
	@NotNull
	private DBUser author;

	@Enumerated(EnumType.STRING)
	@NotNull
	private DBPostPrivacy privacy;

	@NotNull
	private LocalDateTime createdAt;
	private LocalDateTime deletedAt;

	// normal post
	private String content;
	private int replyCount;
	private int repostCount;
	private int quoteCount;

	// reply post
	@ManyToOne(fetch = FetchType.LAZY)
	private DBPost replyTo;

	// repost or quote post
	@ManyToOne(fetch = FetchType.LAZY)
	private DBPost repostOf;
}
