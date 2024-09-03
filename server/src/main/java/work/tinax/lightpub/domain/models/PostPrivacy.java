package work.tinax.lightpub.domain.models;

public enum PostPrivacy {
	PUBLIC, UNLISTED, FOLLOWER, PRIVATE;

	@Override
	public String toString() {
		switch (this) {
		case PUBLIC:
			return "public";
		case UNLISTED:
			return "unlisted";
		case FOLLOWER:
			return "follower";
		case PRIVATE:
			return "private";
		default:
			throw new RuntimeException();
		}
	}

	public static PostPrivacy parse(String text) {
		switch (text) {
		case "public":
			return PUBLIC;
		case "unlisted":
			return UNLISTED;
		case "follower":
			return FOLLOWER;
		case "private":
			return PRIVATE;
		default:
			throw new IllegalArgumentException("unknown PostPrivacy: " + text);
		}
	}
}
