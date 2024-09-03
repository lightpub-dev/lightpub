package work.tinax.lightpub.domain.models;

import java.util.List;

public record PostCounter(int replyCount, int repostCount, int quoteCount, List<PostReactionEntry> reactions) {

}
