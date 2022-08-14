package org.libsdl.nes;

import android.os.Bundle;
import android.util.Log;
import android.view.KeyEvent;
import android.view.View;
import android.widget.Button;

import org.libsdl.app.SDLActivity;
import org.libsdl.nes.view.JoystickView;

import java.io.IOException;
import java.io.InputStream;

public class NESActivity extends SDLActivity {
    private static final String TAG = "NESActivity";

    private int mPrevDirection = 0;

    @Override
    protected String[] getLibraries() {
        return new String[]{"SDL2", "rust_nes"};
    }

    @Override
    protected String getMainFunction() {
        return "android_main";
    }

    @Override
    protected String[] getArguments() {
        String[] ret = new String[2];
        try {
            InputStream is = getAssets().open("mario.nes");
            byte[] bytes = new byte[is.available()];
            is.read(bytes);
            ret[0] = new String(bytes);
        } catch (IOException e) {
            e.printStackTrace();
        }
        getAssets().getLocales();
        ret[1] = getApplicationInfo().sourceDir;
        return ret;
    }


    private View.OnClickListener listenerFactory(int keycode) {
        return v -> {
            SDLActivity.onNativeKeyDown(keycode);
//            v.post(() -> SDLActivity.onNativeKeyUp(keycode));
            v.postDelayed(() -> SDLActivity.onNativeKeyUp(keycode), 10);
        };
    }

    private int[] direction2KeyCodes(int dir) {
        switch (dir) {
            case 1: // right
                return new int[]{KeyEvent.KEYCODE_D};
            case 2: // right-top
                return new int[]{KeyEvent.KEYCODE_D, KeyEvent.KEYCODE_W};
            case 3: // top
                return new int[]{KeyEvent.KEYCODE_W};
            case 4: // left-top
                return new int[]{KeyEvent.KEYCODE_W, KeyEvent.KEYCODE_A};
            case 5: // left
                return new int[]{KeyEvent.KEYCODE_A};
            case 6: // left-down
                return new int[]{KeyEvent.KEYCODE_A, KeyEvent.KEYCODE_S};
            case 7: // down
                return new int[]{KeyEvent.KEYCODE_S};
            case 8: // right-down
                return new int[]{KeyEvent.KEYCODE_S, KeyEvent.KEYCODE_D};
            default: // 0
                return new int[]{};
        }
    }

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        getLayoutInflater().inflate(R.layout.joystacklayout, mLayout, true);
        JoystickView view = findViewById(R.id.joystick);
        view.setOnJoystickMoveListener((angle, power, direction) -> {
            Log.i(TAG, "direction: " + direction);
            int[] keys = direction2KeyCodes(direction);
            int[] oldKeys = direction2KeyCodes(mPrevDirection);

            for (int key : oldKeys) {
                SDLActivity.onNativeKeyUp(key);
            }
            for (int key : keys) {
                SDLActivity.onNativeKeyDown(key);
            }
            mPrevDirection = direction;
        }, 10);
        Button btn_start = findViewById(R.id.btn_start);
        btn_start.setOnClickListener(listenerFactory(KeyEvent.KEYCODE_ENTER));

        Button btn_select = findViewById(R.id.btn_select);
        btn_select.setOnClickListener(listenerFactory(KeyEvent.KEYCODE_SHIFT_RIGHT));

        Button btn_a = findViewById(R.id.btn_a);
        btn_a.setOnClickListener(listenerFactory(KeyEvent.KEYCODE_J));

        Button btn_b = findViewById(R.id.btn_b);
        btn_b.setOnClickListener(listenerFactory(KeyEvent.KEYCODE_K));
    }

    public int dp2px(int dp) {
        float density = getResources().getDisplayMetrics().density;
        return (int) (density * dp + 0.5f);
    }
}
