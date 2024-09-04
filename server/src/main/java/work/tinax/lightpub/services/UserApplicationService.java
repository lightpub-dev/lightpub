package work.tinax.lightpub.services;

import java.util.Objects;
import java.util.Optional;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Service;

import com.auth0.jwt.JWT;
import com.auth0.jwt.algorithms.Algorithm;
import com.auth0.jwt.exceptions.JWTCreationException;

import jakarta.transaction.Transactional;
import work.tinax.lightpub.db.repositories.AuthTokenRepository;
import work.tinax.lightpub.db.repositories.UserRepository;
import work.tinax.lightpub.domain.models.AuthTokenService;
import work.tinax.lightpub.domain.models.UserService;
import work.tinax.lightpub.domain.models.factory.AuthTokenFactory;
import work.tinax.lightpub.domain.models.factory.UserFactory;
import work.tinax.lightpub.utils.UuidUtils;

@Service
public class UserApplicationService {

	private final UserService userService;
	private final UserRepository userRepository;
	private final UserFactory userFactory;
	private final AuthTokenFactory authTokenFactory;
	private final AuthTokenService authTokenService;
	private final AuthTokenRepository authTokenRepository;
	private final AuthApplicationService authApplicationService;

	@Autowired
	public UserApplicationService(UserService userService, UserRepository userRepository, UserFactory userFactory,
			AuthTokenFactory authTokenFactory, AuthTokenService authTokenService,
			AuthTokenRepository authTokenRepository, AuthApplicationService authApplicationService) {
		this.userService = userService;
		this.userRepository = userRepository;
		this.userFactory = userFactory;
		this.authTokenFactory = authTokenFactory;
		this.authTokenService = authTokenService;
		this.authTokenRepository = authTokenRepository;
		this.authApplicationService = authApplicationService;
	}

	@Transactional
	public UserRegisterResult register(String username, String plainPassword, String nickname) {
		var newUser = userFactory.create(username, nickname, plainPassword);

		// duplication check
		if (userService.duplicateCheck(username, null)) {
			throw new RuntimeException("user duplicated");
		}

		var dbUser = userService.toDBUser(newUser);
		userRepository.save(dbUser);

		return new UserRegisterResult(Objects.requireNonNull(UuidUtils.trim(newUser.getId().getId())));
	}

	@Transactional
	public Optional<UserLoginResult> login(String username, String password) {
		var dbUser = userRepository.findByUsernameAndHostname(username, null);
		if (dbUser == null) {
			return Optional.empty();
		}

		if (dbUser.getBpasswd() == null) {
			return Optional.empty();
		}

		var user = userService.fromDBUser(dbUser);

		if (!userService.validatePassword(user, password)) {
			return Optional.empty();
		}

		// create a new token
		var token = authApplicationService.createToken(UuidUtils.trim(user.getId().getId()));

		return Optional.of(new UserLoginResult(token));
	}
}
