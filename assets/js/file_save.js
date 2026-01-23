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
