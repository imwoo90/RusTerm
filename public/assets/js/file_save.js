// File saving utility using File System Access API
export async function save_stream_to_disk(stream) {
    try {
        // Check if File System Access API is supported
        if (!window.showSaveFilePicker) {
            alert('Your browser does not support streaming save. Please use Chrome/Edge.');
            return;
        }

        const handle = await window.showSaveFilePicker({
            suggestedName: 'serial_log.txt',
            types: [{
                description: 'Text Files',
                accept: { 'text/plain': ['.txt'] }
            }],
        });

        const writable = await handle.createWritable();
        await stream.pipeTo(writable);
        console.log('Stream save completed');
    } catch (err) {
        if (err.name !== 'AbortError') {
            console.error('Save failed:', err);
            alert('Save failed: ' + err);
        }
    }
}

export async function save_terminal_history(terminal) {
    try {
        if (!window.showSaveFilePicker) {
            alert('Your browser does not support file saving. Please use Chrome/Edge.');
            return;
        }

        const buffer = terminal.buffer.active;
        let content = '';
        for (let i = 0; i < buffer.length; i++) {
            const line = buffer.getLine(i);
            if (line) {
                content += line.translateToString(true) + '\n';
            }
        }

        const handle = await window.showSaveFilePicker({
            suggestedName: 'terminal_history.txt',
            types: [{
                description: 'Text Files',
                accept: { 'text/plain': ['.txt'] }
            }],
        });

        const writable = await handle.createWritable();
        await writable.write(content);
        await writable.close();
        console.log('Terminal history save completed');
    } catch (err) {
        if (err.name !== 'AbortError') {
            const errorMsg = err.message || String(err);
            console.error('Save failed:', errorMsg);
            alert('Save failed: ' + errorMsg);
        }
    }
}
