(function () {
    const output = document.getElementById('console-output');
    const input = document.getElementById('input');

    // ----- Utilities ----- //
    function printLine(text, cls = '') {
        const div = document.createElement('div');
        div.className = 'line ' + cls;
        div.textContent = text;
        output.appendChild(div);
        output.scrollTop = output.scrollHeight;
    }

    // ----- Command Execution ----- //
    window.smnconsole = {
        async runCommand(cmd) {
            try {
                const response = await fetch('/api/cmd', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ command: cmd.replace(/\r?\n/g, '\\n') })
                });

                if (!response.ok) {
                    return `Error: ${response.status} ${response.statusText}`;
                }

                const json = await response.json();
                return json.message || '(no message)';
            } catch (err) {
                return `Request failed: ${err.message}`;
            }
        }
    };

    // ----- Input Behavior ----- //

    // Dynamic resize
    input.addEventListener('input', () => {
        input.style.height = 'auto';
        input.style.height = input.scrollHeight + 'px';
    });

    // Command History
    const history = [];
    let historyIndex = -1;

    input.addEventListener('keydown', async (e) => {
        // Submit command
        if (e.key === 'Enter' && !e.shiftKey && !e.ctrlKey) {
            e.preventDefault();
            if (input.value.trim() === '') return;

            const command = input.value.trim();
            printLine(`> ${command}`, 'cmd');

            // Store in history
            history.push(command);
            historyIndex = history.length;

            const result = await window.smnconsole.runCommand(command);
            printLine(result, 'result');

            input.value = '';
            input.style.height = 'auto';
        }

        // Multiline insert (Ctrl+Enter)
        else if (e.key === 'Enter' && e.ctrlKey) {
            e.preventDefault();
            const start = input.selectionStart;
            const end = input.selectionEnd;
            const value = input.value;
            input.value = value.slice(0, start) + '\n' + value.slice(end);
            input.selectionStart = input.selectionEnd = start + 1;
            input.dispatchEvent(new Event('input'));
        }

        // History navigation (Ctrl+Up / Ctrl+Down)
        else if (e.ctrlKey && e.key === 'ArrowUp') {
            e.preventDefault();
            if (history.length === 0) return;

            if (historyIndex > 0) {
                historyIndex--;
            } else {
                historyIndex = 0;
            }

            input.value = history[historyIndex];
            input.dispatchEvent(new Event('input'));
        }

        else if (e.ctrlKey && e.key === 'ArrowDown') {
            e.preventDefault();
            if (history.length === 0) return;

            if (historyIndex < history.length - 1) {
                historyIndex++;
                input.value = history[historyIndex];
            } else {
                historyIndex = history.length;
                input.value = '';
            }

            input.dispatchEvent(new Event('input'));
        }
    });

    // ----- Startup Message ----- //
    printLine("Charmline Developer Console v0.1");
    printLine("Type a command and press Enter.");
})();
