package com.brave.adblock;

public class AdBlockClient {
    private long nativeThis;

    public enum FilterOption {
        UNKNOWN, SCRIPT, CSS, OBJECT, IMAGE, FONT, DOCUMENT, MEDIA, SUB_FRAME, WEB_SOCKET, XML_HTTP_REQUEST
    }

    private native void deinit();
    public native boolean loadRules(String input);
    public native boolean serialize(String fileName);
    public native boolean deserialize(String fileName);
    public native boolean matches(String urlToCheck, int filterOption, String sourceUrl);

    public boolean matches(String urlToCheck, FilterOption filterOption, String sourceUrl) {
        return matches(urlToCheck, filterOption.ordinal(), sourceUrl);
    }

    @Override
    protected void finalize() throws Throwable {
        deinit();
        super.finalize();
    }

    static {
        System.loadLibrary("adblock_rs");
    }
}
