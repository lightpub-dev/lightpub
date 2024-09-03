package work.tinax.lightpub.domain.models;

import java.time.LocalDateTime;

public class User {
	private UserId id;

	private Username username;
	private String hostname;
	private String bpasswd;
	private Nickname nickname;
	private String bio;
	private URL url;
	private URL inbox;
	private URL sharedInbox;
	private URL outbox;
	private URL followings;
	private URL followers;

	private String privateKey;
	private String publicKey;
	private LocalDateTime createdAt;

	public User(UserId id, Username username, String hostname, String bpasswd, Nickname nickname, String bio, URL url,
			URL inbox, URL sharedInbox, URL outbox, URL followings, URL followers, String privateKey, String publicKey,
			LocalDateTime createdAt) {
		this.id = id;
		this.username = username;
		this.hostname = hostname;
		this.bpasswd = bpasswd;
		this.nickname = nickname;
		this.bio = bio;
		this.url = url;
		this.inbox = inbox;
		this.sharedInbox = sharedInbox;
		this.outbox = outbox;
		this.followings = followings;
		this.followers = followers;
		this.privateKey = privateKey;
		this.publicKey = publicKey;
		this.createdAt = createdAt;
	}

}
