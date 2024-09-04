package work.tinax.lightpub.domain.models;

import java.util.UUID;

import org.eclipse.jdt.annotation.Nullable;

import work.tinax.lightpub.utils.UuidUtils;

public class UserId {
	private UUID id;

	public UUID getId() {
		return id;
	}

	public UserId(UUID id) {
		this.id = id;
	}

	public static UserId parse(String s) {
		return new UserId(UuidUtils.fromTrimmed(s));
	}

	@Override
	public boolean equals(@Nullable Object o) {
		if (o instanceof UserId u) {
			return id.equals(u.id);
		}
		return false;
	}

	@Override
	public int hashCode() {
		return id.hashCode();
	}
}
