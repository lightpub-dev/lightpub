package work.tinax.lightpub.domain.models.factory;

import java.util.Optional;

import org.eclipse.jdt.annotation.NonNullByDefault;
import org.springframework.security.crypto.bcrypt.BCrypt;
import org.springframework.stereotype.Service;

import com.github.f4b6a3.uuid.UuidCreator;

import work.tinax.lightpub.domain.models.Nickname;
import work.tinax.lightpub.domain.models.User;
import work.tinax.lightpub.domain.models.UserId;
import work.tinax.lightpub.domain.models.Username;
import work.tinax.lightpub.utils.ClockUtils;

@NonNullByDefault
@Service
public class DefaultUserFactory implements UserFactory {

	@Override
	public User create(String username, String nickname, String passwd) {
		@SuppressWarnings("null")
		var userId = new UserId(UuidCreator.getTimeOrderedEpoch());
		var usernameObj = new Username(username);
		var bpasswd = BCrypt.hashpw(passwd, BCrypt.gensalt());
		var nicknameObj = new Nickname(nickname);
		var createdAt = ClockUtils.now();
		return new User(userId, usernameObj, Optional.empty(), Optional.of(bpasswd), nicknameObj, "", Optional.empty(),
				Optional.empty(), Optional.empty(), Optional.empty(), Optional.empty(),
				Optional.empty(), Optional.empty(),
				Optional.empty(), createdAt);
	}

}
