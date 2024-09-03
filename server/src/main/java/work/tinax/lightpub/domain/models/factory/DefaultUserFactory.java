package work.tinax.lightpub.domain.models.factory;

import org.eclipse.jdt.annotation.NonNullByDefault;
import org.springframework.security.crypto.bcrypt.BCrypt;

import com.github.f4b6a3.uuid.UuidCreator;

import work.tinax.lightpub.domain.models.Nickname;
import work.tinax.lightpub.domain.models.User;
import work.tinax.lightpub.domain.models.UserId;
import work.tinax.lightpub.domain.models.Username;
import work.tinax.lightpub.utils.ClockUtils;

@NonNullByDefault
public class DefaultUserFactory implements UserFactory {

	@Override
	public User create(String username, String nickname, String passwd) {
		@SuppressWarnings("null")
		var userId = new UserId(UuidCreator.getTimeOrderedEpoch());
		var usernameObj = new Username(username);
		var bpasswd = BCrypt.hashpw(passwd, BCrypt.gensalt());
		var nicknameObj = new Nickname(nickname);
		var createdAt = ClockUtils.now();
		return new User(userId, usernameObj, null, bpasswd, nicknameObj, "", null, null, null, null, null, null, null,
				null, createdAt);
	}

}
