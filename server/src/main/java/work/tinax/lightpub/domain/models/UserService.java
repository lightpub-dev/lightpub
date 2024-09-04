package work.tinax.lightpub.domain.models;

import java.util.Optional;

import org.eclipse.jdt.annotation.Nullable;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.security.crypto.bcrypt.BCrypt;
import org.springframework.stereotype.Service;

import jakarta.transaction.Transactional;
import work.tinax.lightpub.db.models.DBUser;
import work.tinax.lightpub.db.repositories.UserRepository;
import work.tinax.lightpub.utils.UuidUtils;

@Service
public class UserService {

	private final UserRepository userRepository;

	@Autowired
	public UserService(UserRepository userRepository) {
		this.userRepository = userRepository;
	}

	@Transactional
	public boolean duplicateCheck(String username, @Nullable String hostname) {
		var user = userRepository.findByUsernameAndHostname(username, hostname);
		return user != null;
	}

	@SuppressWarnings("null")
	public DBUser toDBUser(User u) {
		var d = new DBUser();

		d.setUserId(UuidUtils.trim(u.getId().getId()));
		d.setUsername(u.getUsername().getUsername());
		d.setHostname(u.getHostname().orElse(null));
		d.setBpasswd(u.getBpasswd().orElse(null));
		d.setNickname(u.getNickname().getNickname());
		d.setBio(u.getBio());
		u.getUrl().ifPresent(n -> d.setUrl(n.getUrl()));
		u.getInbox().ifPresent(i -> d.setInbox(i.getUrl()));
		u.getSharedInbox().ifPresent(i -> d.setSharedInbox(i.getUrl()));
		u.getOutbox().ifPresent(i -> d.setOutbox(i.getUrl()));
		u.getFollowings().ifPresent(i -> d.setFollowings(i.getUrl()));
		u.getFollowers().ifPresent(i -> d.setFollowers(i.getUrl()));
		u.getPrivateKey().ifPresent(i -> d.setPrivateKey(i));
		u.getPublicKey().ifPresent(i -> d.setPublicKey(i));
		d.setCreatedAt(u.getCreatedAt());

		return d;
	}

	public boolean validatePassword(User u, String passwd) {
		return u.getBpasswd().map(p -> {
			return BCrypt.checkpw(passwd, p);
		}).orElse(false);
	}

	public User fromDBUser(DBUser d) {
		return new User(UserId.parse(d.getUserId()), new Username(d.getUsername()),
				Optional.ofNullable(d.getHostname()), Optional.ofNullable(d.getBpasswd()),
				new Nickname(d.getNickname()), d.getBio(), Optional.ofNullable(URL.parse(d.getUrl())),
				Optional.ofNullable(URL.parse(d.getInbox())), Optional.ofNullable(URL.parse(d.getSharedInbox())),
				Optional.ofNullable(URL.parse(d.getOutbox())), Optional.ofNullable(URL.parse(d.getFollowings())),
				Optional.ofNullable(URL.parse(d.getFollowers())), Optional.ofNullable(d.getPrivateKey()),
				Optional.ofNullable(d.getPublicKey()), d.getCreatedAt());
	}
}
