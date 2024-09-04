package work.tinax.lightpub.web;

public class InvalidAuthException extends LightpubException {
    @Override
    public int getStatus() {
        return 401;
    }

    @Override
    public String getText() {
        return "invalid authentication";
    }
}
