package work.tinax.lightpub.domain.models;

import java.time.LocalDateTime;

import org.eclipse.jdt.annotation.Nullable;

public class User {
	private UserId id;

	private Username username;
	@Nullable
	private String hostname;
	@Nullable
	private String bpasswd;
	private Nickname nickname;
	private String bio;
	@Nullable
	private URL url;
	@Nullable
	private URL inbox;
	@Nullable
	private URL sharedInbox;
	@Nullable
	private URL outbox;
	@Nullable
	private URL followings;
	@Nullable
	private URL followers;

	@Nullable
	private String privateKey;
	@Nullable
	private String publicKey;
	private LocalDateTime createdAt;

	public User(UserId id, Username username, @Nullable String hostname, @Nullable String bpasswd, Nickname nickname,
			String bio, @Nullable URL url, @Nullable URL inbox, @Nullable URL sharedInbox, @Nullable URL outbox,
			@Nullable URL followings, @Nullable URL followers, @Nullable String privateKey, @Nullable String publicKey,
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

	public LocalDateTime getCreatedAt() {
		return createdAt;
	}

	public void setCreatedAt(LocalDateTime createdAt) {
		this.createdAt = createdAt;
	}

	@Nullable
	public String getPublicKey() {
		return publicKey;
	}

	public void setPublicKey(String publicKey) {
		this.publicKey = publicKey;
	}

	@Nullable
	public String getPrivateKey() {
		return privateKey;
	}

	public void setPrivateKey(String privateKey) {
		this.privateKey = privateKey;
	}

	@Nullable
	public URL getFollowers() {
		return followers;
	}

	public void setFollowers(URL followers) {
		this.followers = followers;
	}

	@Nullable
	public URL getFollowings() {
		return followings;
	}

	public void setFollowings(URL followings) {
		this.followings = followings;
	}

	@Nullable
	public URL getOutbox() {
		return outbox;
	}

	public void setOutbox(URL outbox) {
		this.outbox = outbox;
	}

	@Nullable
	public URL getSharedInbox() {
		return sharedInbox;
	}

	public void setSharedInbox(URL sharedInbox) {
		this.sharedInbox = sharedInbox;
	}

	@Nullable
	public URL getInbox() {
		return inbox;
	}

	public void setInbox(URL inbox) {
		this.inbox = inbox;
	}

	@Nullable
	public URL getUrl() {
		return url;
	}

	public void setUrl(URL url) {
		this.url = url;
	}

	public String getBio() {
		return bio;
	}

	public void setBio(String bio) {
		this.bio = bio;
	}

	public Nickname getNickname() {
		return nickname;
	}

	public void setNickname(Nickname nickname) {
		this.nickname = nickname;
	}

	@Nullable
	public String getBpasswd() {
		return bpasswd;
	}

	public void setBpasswd(String bpasswd) {
		this.bpasswd = bpasswd;
	}

	@Nullable
	public String getHostname() {
		return hostname;
	}

	public void setHostname(String hostname) {
		this.hostname = hostname;
	}

	public Username getUsername() {
		return username;
	}

	public void setUsername(Username username) {
		this.username = username;
	}

	public UserId getId() {
		return id;
	}
}
