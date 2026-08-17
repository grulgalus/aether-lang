package com.aether.studio;

import android.app.Activity;
import android.os.Bundle;
import android.widget.Button;
import android.widget.EditText;
import android.widget.TextView;

public class MainActivity extends Activity {
    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_main);

        EditText codeInput = findViewById(R.id.codeInput);
        Button runButton = findViewById(R.id.runButton);
        TextView outputView = findViewById(R.id.outputView);

        codeInput.setText("// Nativni Java build!\nlet uvitan = \"Ahoj svete!\"\nprint(uvitan)\nprint(\"10 * 10 =\")\nprint(10 * 10)");

        runButton.setOnClickListener(v -> {
            outputView.setText("Kompiluji přes jádro v Rustu...");
            try {
                // Zavoláme Rust přímo z Javy!
                String result = AetherCore.runCode(codeInput.getText().toString());
                outputView.setText(result);
            } catch (Exception e) {
                outputView.setText("Kritická chyba spojení: " + e.getMessage());
            }
        });
    }
}
