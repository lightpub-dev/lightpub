package work.tinax.lightpub.db.models;

import java.time.LocalDateTime;

import jakarta.persistence.Column;
import jakarta.persistence.Entity;
import jakarta.persistence.Id;
import jakarta.persistence.Table;
import jakarta.validation.constraints.NotNull;

@Entity
@Table(name = "\"user\"")
@SuppressWarnings("unused")
public class DBUser {
	@Id
	@Column(name = "id")
	private String userId;

	@NotNull
	private String username;
	private String hostname;
	private String bpasswd;
	@NotNull
	private String nickname;
	@NotNull
	private String bio;

	private String url;
	private String inbox;
	private String sharedInbox;
	private String outbox;
	private String followings;
	private String followers;

	private String privateKey;
	private String publicKey;
	@NotNull
	private LocalDateTime createdAt;

	public String getUserId() {
		return userId;
	}

	public void setUserId(String userId) {
		this.userId = userId;
	}

	public String getUsername() {
		return username;
	}

	public void setUsername(String username) {
		this.username = username;
	}

	public String getHostname() {
		return hostname;
	}

	public void setHostname(String hostname) {
		this.hostname = hostname;
	}

	public String getBpasswd() {
		return bpasswd;
	}

	public void setBpasswd(String bpasswd) {
		this.bpasswd = bpasswd;
	}

	public String getNickname() {
		return nickname;
	}

	public void setNickname(String nickname) {
		this.nickname = nickname;
	}

	public String getBio() {
		return bio;
	}

	public void setBio(String bio) {
		this.bio = bio;
	}

	public String getUrl() {
		return url;
	}

	public void setUrl(String url) {
		this.url = url;
	}

	public String getInbox() {
		return inbox;
	}

	public void setInbox(String inbox) {
		this.inbox = inbox;
	}

	public String getSharedInbox() {
		return sharedInbox;
	}

	public void setSharedInbox(String sharedInbox) {
		this.sharedInbox = sharedInbox;
	}

	public String getOutbox() {
		return outbox;
	}

	public void setOutbox(String outbox) {
		this.outbox = outbox;
	}

	public String getFollowings() {
		return followings;
	}

	public void setFollowings(String followings) {
		this.followings = followings;
	}

	public String getFollowers() {
		return followers;
	}

	public void setFollowers(String followers) {
		this.followers = followers;
	}

	public String getPrivateKey() {
		return privateKey;
	}

	public void setPrivateKey(String privateKey) {
		this.privateKey = privateKey;
	}

	public String getPublicKey() {
		return publicKey;
	}

	public void setPublicKey(String publicKey) {
		this.publicKey = publicKey;
	}

	public LocalDateTime getCreatedAt() {
		return createdAt;
	}

	public void setCreatedAt(LocalDateTime createdAt) {
		this.createdAt = createdAt;
	}
}
