package com.brave.adblock;

import android.net.Uri;
import android.webkit.WebResourceRequest;

import java.util.Locale;

public class Utils {
    private static native String greeting(final String pattern);

    public static AdBlockClient.FilterOption mapRequestToFilterOption(WebResourceRequest webResourceRequest) {
        String acceptHeader = webResourceRequest.getRequestHeaders().get("Accept");
        if (acceptHeader != null) {
            if (acceptHeader.contains("image/")) {
                return AdBlockClient.FilterOption.IMAGE;
            }
            if (acceptHeader.contains("/css")) {
                return AdBlockClient.FilterOption.CSS;
            }
            if (acceptHeader.contains("javascript")) {
                return AdBlockClient.FilterOption.SCRIPT;
            }
            if (acceptHeader.contains("video/")) {
                return AdBlockClient.FilterOption.OBJECT;
            }
        }

        Uri url = webResourceRequest.getUrl();
        if (url != null) {
            if (uriHasExtension(url,"css")) {
                return AdBlockClient.FilterOption.CSS;
            }
            if (uriHasExtension(url,"js")) {
                return AdBlockClient.FilterOption.SCRIPT;
            }
            if (uriHasExtension(url,"png", "jpg", "jpeg", "webp", "svg", "gif", "bmp", "tiff")) {
                return AdBlockClient.FilterOption.IMAGE;
            }
            if (uriHasExtension(url,"mp4", "mov", "avi")) {
                return AdBlockClient.FilterOption.OBJECT;
            }
            if (uriHasExtension(url,"otf", "sfnt", "ttf", "woff", "woff2", "eot")) {
                return AdBlockClient.FilterOption.FONT;
            }
        }

        return AdBlockClient.FilterOption.UNKNOWN;
    }

    public static boolean uriHasExtension(Uri uri, String... values) {
        String path = uri.getPath().toLowerCase(Locale.ROOT);
        for (String ext: values) {
            if (path.endsWith("." + ext)) return true;
        }
        return false;
    }
}
