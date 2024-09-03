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
	private String bio;
	
	@NotNull
	private String url;
	@NotNull
	private String inbox;
	@NotNull
	private String sharedInbox;
	@NotNull
	private String outbox;
	@NotNull
	private String followings;
	@NotNull
	private String followers;
	
	private String privateKey;
	private String publicKey;
	@NotNull
	private LocalDateTime createdAt;
}
