package com.edisonlivings.airbag;

import android.util.Log;
import com.google.gson.Gson;

public class Airbag {
    private static final String TAG = "Airbag";
    private final Gson gson = new Gson();

    static {
        System.loadLibrary("airbag");
    }

    void registerAirbag(AirbagConfig config) {
        String jsonConfig = gson.toJson(config.getSignalConfig());
        Log.d(TAG, "registerAirbag: " + jsonConfig);
        registerNativeAirbag(jsonConfig);
    }

    private native void registerNativeAirbag(String signalConfig);


    static native void sendSignal(int signal);
}
