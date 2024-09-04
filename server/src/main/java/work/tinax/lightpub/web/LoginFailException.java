package work.tinax.lightpub.web;

public class LoginFailException extends LightpubException {
    @Override
    public int getStatus() {
        return 401;
    }

    @Override
    public String getText() {
        return "login failed";
    }
}
