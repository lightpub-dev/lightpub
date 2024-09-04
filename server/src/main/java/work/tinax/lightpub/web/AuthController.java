package work.tinax.lightpub.web;

import org.apache.commons.logging.Log;
import org.apache.commons.logging.LogFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.security.core.context.SecurityContextHolder;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RestController;

import work.tinax.lightpub.services.UserApplicationService;

record UserRegisterResponse(String userId) {
}

record UserRegisterRequest(String username, String nickname, String password) {
}

record UserLoginRequest(String username, String password) {
}

record UserLoginResponse(String token) {
}

record MeResponse(String userId) {
}

@RestController
@RequestMapping("/auth")
public class AuthController {
	private final UserApplicationService userApplicationService;

	@Autowired
	public AuthController(UserApplicationService userApplicationService) {
		this.userApplicationService = userApplicationService;
	}

	private Log log = LogFactory.getLog(AuthController.class);

	@GetMapping("/me")
	public MeResponse hello() {
		var auth = SecurityContextHolder.getContext().getAuthentication();
		return new MeResponse(auth.getName());
	}

	@PostMapping("/register")
	public UserRegisterResponse register(@RequestBody UserRegisterRequest req) {
		var result = userApplicationService.register(req.username(), req.password(), req.nickname());
		return new UserRegisterResponse(result.userId());
	}

	@PostMapping("/login")
	public UserLoginResponse login(@RequestBody UserLoginRequest req) {
		var result = userApplicationService.login(req.username(), req.password());
		if (result.isEmpty()) {
			throw new LoginFailException();
		}
		return new UserLoginResponse(result.get().token());
	}
}
