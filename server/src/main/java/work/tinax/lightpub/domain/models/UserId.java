package work.tinax.lightpub.domain.models;

import java.util.UUID;

public class UserId {
	private UUID id;

	public UUID getId() {
		return id;
	}
	
	public UserId(UUID id) {
		this.id = id;
	}
}
