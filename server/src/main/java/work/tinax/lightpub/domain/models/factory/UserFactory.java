package work.tinax.lightpub.domain.models.factory;

import org.eclipse.jdt.annotation.NonNullByDefault;

import work.tinax.lightpub.domain.models.User;

@NonNullByDefault
public interface UserFactory {
	User create(String username, String nickname, String passwd);
}
