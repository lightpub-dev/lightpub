package work.tinax.lightpub.domain.models;

import org.eclipse.jdt.annotation.Nullable;

public class Nickname {
	private String nickname;

	public Nickname(String nickname) {
		this.nickname = nickname;
	}

	public String getNickname() {
		return nickname;
	}

	@Override
	public boolean equals(@Nullable Object o) {
		if (o instanceof Nickname n) {
			return nickname.equals(n.nickname);
		}
		return false;
	}

	@Override
	public int hashCode() {
		return nickname.hashCode();
	}
}
