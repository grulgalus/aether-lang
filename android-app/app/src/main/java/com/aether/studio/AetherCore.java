package com.aether.studio;

public class AetherCore {
    // Načtení naší dynamické knihovny zkompilované z Rustu
    static { System.loadLibrary("aether_jni"); }
    
    // Spojení Javy s Rust funkcí
    public static native String runCode(String code);
}
