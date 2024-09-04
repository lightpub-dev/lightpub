package work.tinax.lightpub.domain.models.factory;

import org.eclipse.jdt.annotation.NonNullByDefault;
import org.springframework.stereotype.Service;

import com.github.f4b6a3.uuid.UuidCreator;

import work.tinax.lightpub.domain.models.AuthToken;
import work.tinax.lightpub.domain.models.UserId;
import work.tinax.lightpub.utils.ClockUtils;
import work.tinax.lightpub.utils.UuidUtils;

@NonNullByDefault
@Service
public class DefaultAuthTokenFactory implements AuthTokenFactory {

    @Override
    public AuthToken generateNow(UserId userId) {
        var v7 = UuidUtils.trim(UuidCreator.getTimeOrderedEpoch());
        var now = ClockUtils.now();
        return new AuthToken(v7, now, userId);
    }

}
