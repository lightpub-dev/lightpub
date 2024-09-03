package work.tinax.lightpub.domain.models;

import java.time.LocalDateTime;
import java.util.Optional;

public class User {
	private UserId id;

	private Username username;
	private Optional<String> hostname;
	private Optional<String> bpasswd;
	private Nickname nickname;
	private String bio;
	private Optional<URL> url;
	private Optional<URL> inbox;
	private Optional<URL> sharedInbox;
	private Optional<URL> outbox;
	private Optional<URL> followings;
	private Optional<URL> followers;

	private Optional<String> privateKey;
	private Optional<String> publicKey;
	private LocalDateTime createdAt;

	public User(UserId id, Username username, Optional<String> hostname, Optional<String> bpasswd, Nickname nickname,
			String bio, Optional<URL> url, Optional<URL> inbox, Optional<URL> sharedInbox, Optional<URL> outbox,
			Optional<URL> followings, Optional<URL> followers, Optional<String> privateKey, Optional<String> publicKey,
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

	public UserId getId() {
		return id;
	}

	public void setId(UserId id) {
		this.id = id;
	}

	public Username getUsername() {
		return username;
	}

	public void setUsername(Username username) {
		this.username = username;
	}

	public Optional<String> getHostname() {
		return hostname;
	}

	public void setHostname(Optional<String> hostname) {
		this.hostname = hostname;
	}

	public Optional<String> getBpasswd() {
		return bpasswd;
	}

	public void setBpasswd(Optional<String> bpasswd) {
		this.bpasswd = bpasswd;
	}

	public Nickname getNickname() {
		return nickname;
	}

	public void setNickname(Nickname nickname) {
		this.nickname = nickname;
	}

	public String getBio() {
		return bio;
	}

	public void setBio(String bio) {
		this.bio = bio;
	}

	public Optional<URL> getUrl() {
		return url;
	}

	public void setUrl(Optional<URL> url) {
		this.url = url;
	}

	public Optional<URL> getInbox() {
		return inbox;
	}

	public void setInbox(Optional<URL> inbox) {
		this.inbox = inbox;
	}

	public Optional<URL> getSharedInbox() {
		return sharedInbox;
	}

	public void setSharedInbox(Optional<URL> sharedInbox) {
		this.sharedInbox = sharedInbox;
	}

	public Optional<URL> getOutbox() {
		return outbox;
	}

	public void setOutbox(Optional<URL> outbox) {
		this.outbox = outbox;
	}

	public Optional<URL> getFollowings() {
		return followings;
	}

	public void setFollowings(Optional<URL> followings) {
		this.followings = followings;
	}

	public Optional<URL> getFollowers() {
		return followers;
	}

	public void setFollowers(Optional<URL> followers) {
		this.followers = followers;
	}

	public Optional<String> getPrivateKey() {
		return privateKey;
	}

	public void setPrivateKey(Optional<String> privateKey) {
		this.privateKey = privateKey;
	}

	public Optional<String> getPublicKey() {
		return publicKey;
	}

	public void setPublicKey(Optional<String> publicKey) {
		this.publicKey = publicKey;
	}

	public LocalDateTime getCreatedAt() {
		return createdAt;
	}

	public void setCreatedAt(LocalDateTime createdAt) {
		this.createdAt = createdAt;
	}

}
