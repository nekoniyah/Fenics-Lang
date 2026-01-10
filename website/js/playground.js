// Playground functionality
document.addEventListener("DOMContentLoaded", () => {
  const codeEditor = document.getElementById("codeEditor");
  const output = document.getElementById("output");
  const runBtn = document.getElementById("runBtn");
  const clearBtn = document.getElementById("clearBtn");

  // Mock Fenics interpreter (since we're in a browser)
  // In a real implementation, this would connect to a backend service
  function interpretFenics(code) {
    // This is a mock interpreter for demonstration
    // It simulates some basic output
    const lines = code.split("\n");
    const results = [];

    results.push(
      "// Fenics Interpreter Output\n// (Mock execution for demo purposes)\n"
    );

    // Simple pattern matching for demo
    let inFunction = false;
    let functionName = "";

    lines.forEach((line, index) => {
      const trimmed = line.trim();

      // Detect print statements
      if (trimmed.includes("print(")) {
        const match = trimmed.match(/print\((.*?)\)/);
        if (match) {
          let content = match[1];
          // Simple string interpolation handling
          content = content.replace(/["']/g, "");
          content = content.replace(/#{(\w+)}/g, (_, variable) => {
            // For demo, return placeholder values
            const mockValues = {
              name: "Alice",
              age: "25",
              i: "0",
              result: "8",
              user: "World",
              count: "0",
            };
            return mockValues[variable] || variable;
          });
          results.push(content);
        }
      }

      // Detect function definitions
      if (trimmed.startsWith("fn ")) {
        const match = trimmed.match(/fn (\w+)/);
        if (match) {
          functionName = match[1];
          inFunction = true;
        }
      }

      // Detect loop patterns
      if (trimmed.startsWith("for ") && trimmed.includes("in")) {
        const match = trimmed.match(/for \w+ in (\d+)\.\.(\d+)/);
        if (match) {
          const start = parseInt(match[1]);
          const end = parseInt(match[2]);
          results.push(`// Loop executing from ${start} to ${end - 1}`);
        }
      }
    });

    // Check for specific example patterns
    if (code.includes("fibonacci")) {
      results.push("\nFibonacci sequence:");
      const fibs = [0, 1, 1, 2, 3, 5, 8, 13, 21, 34];
      for (let i = 0; i < 10; i++) {
        results.push(`F(${i}) = ${fibs[i]}`);
      }
    } else if (code.includes("Hello")) {
      results.push("Hello, World!");
    } else if (results.length <= 1) {
      results.push("\n✓ Code executed successfully!");
      results.push(
        "(Note: This is a demo playground. Full interpreter available via CLI)"
      );
    }

    return results.join("\n");
  }

  // Run button handler
  runBtn.addEventListener("click", () => {
    const code = codeEditor.value;

    if (!code.trim()) {
      output.textContent = "Error: No code to execute";
      output.style.color = "var(--error-color)";
      return;
    }

    runBtn.disabled = true;
    runBtn.textContent = "⟳ Running...";
    output.textContent = "Executing...";
    output.style.color = "var(--text-muted)";

    // Simulate execution delay
    setTimeout(() => {
      try {
        const result = interpretFenics(code);
        output.textContent = result;
        output.style.color = "var(--success-color)";
      } catch (error) {
        output.textContent = `Error: ${error.message}`;
        output.style.color = "var(--error-color)";
      } finally {
        runBtn.disabled = false;
        runBtn.textContent = "▶ Run";
      }
    }, 500);
  });

  // Clear button handler
  clearBtn.addEventListener("click", () => {
    output.textContent = 'Output cleared. Click "Run" to execute your code...';
    output.style.color = "var(--text-muted)";
  });

  // Add keyboard shortcut (Ctrl/Cmd + Enter to run)
  codeEditor.addEventListener("keydown", (e) => {
    if ((e.ctrlKey || e.metaKey) && e.key === "Enter") {
      e.preventDefault();
      runBtn.click();
    }
  });

  // Tab key support in textarea
  codeEditor.addEventListener("keydown", (e) => {
    if (e.key === "Tab") {
      e.preventDefault();
      const start = codeEditor.selectionStart;
      const end = codeEditor.selectionEnd;
      const value = codeEditor.value;

      codeEditor.value =
        value.substring(0, start) + "    " + value.substring(end);
      codeEditor.selectionStart = codeEditor.selectionEnd = start + 4;
    }
  });

  // Auto-save to localStorage
  codeEditor.addEventListener("input", () => {
    localStorage.setItem("fenics-playground-code", codeEditor.value);
  });

  // Load saved code on page load
  const savedCode = localStorage.getItem("fenics-playground-code");
  if (savedCode) {
    codeEditor.value = savedCode;
  }
});
