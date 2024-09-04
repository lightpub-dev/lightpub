package work.tinax.lightpub.web;

public class LightpubException extends RuntimeException {
    public int getStatus() {
        return 500;
    }

    public String getText() {
        return "Internal Server Error";
    }
}
