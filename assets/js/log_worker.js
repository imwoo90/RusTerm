import init, { LogProcessor } from "/wasm/serial_monitor.js";

let processor, lastNotify = 0, pending = null, currentFile = null;

const notify = (count) => {
    if (Date.now() - lastNotify > 50) {
        lastNotify = Date.now();
        self.postMessage({ type: 'TOTAL_LINES', data: count });
        if (pending) clearTimeout(pending), pending = null;
    } else if (!pending) pending = setTimeout(() => notify(processor.get_line_count()), 50);
};

const getFiles = async (root) => {
    const files = [];
    for await (const [n, h] of root.entries()) if (n.startsWith('logs_') && n.endsWith('.txt')) files.push([n, h]);
    return files.sort((a, b) => parseInt(b[0].split('_')[1]) - parseInt(a[0].split('_')[1])); // Descending
};

const newSession = async (root, cleanup = false) => {
    if (cleanup && currentFile) try { await root.removeEntry(currentFile); } catch (e) { }
    currentFile = `logs_${Date.now()}.txt`;
    const h = await (await root.getFileHandle(currentFile, { create: true })).createSyncAccessHandle();
    processor.set_sync_handle(h);
    processor.clear();
    return h;
};

(async () => {
    try {
        await init();
        processor = new LogProcessor();
        const root = await navigator.storage.getDirectory();

        // Recover latest or start new
        const files = await getFiles(root);
        if (files.length > 0) {
            currentFile = files[0][0];
            processor.set_sync_handle(await files[0][1].createSyncAccessHandle()); // Triggers rebuild
            for (let i = 1; i < files.length; i++) try { await root.removeEntry(files[i][0]); } catch (e) { } // delete stale
        } else {
            await newSession(root);
        }

        self.postMessage({ type: 'INITIALIZED' });
        if (processor.get_line_count() > 0) self.postMessage({ type: 'TOTAL_LINES', data: processor.get_line_count() });

        self.onmessage = async ({ data: { type, data } }) => {
            try {
                if (type === 'NEW_SESSION') { await newSession(root, true); self.postMessage({ type: 'TOTAL_LINES', data: 0 }); }
                else if (type === 'APPEND_CHUNK') { processor.append_chunk(data.chunk, data.is_hex); notify(processor.get_line_count()); }
                else if (type === 'REQUEST_WINDOW') self.postMessage({ type: 'LOG_WINDOW', data: { startLine: data.startLine, lines: processor.request_window(data.startLine, data.count) } });
                else if (type === 'SEARCH_LOGS') { processor.search_logs(data.query, data.match_case, data.use_regex, data.invert); notify(processor.get_line_count()); }
                else if (type === 'EXPORT_LOGS') { const s = processor.export_logs(!(data?.include_timestamp === false)); self.postMessage({ type: 'EXPORT_STREAM', stream: s }, [s]); }
                else if (type === 'CLEAR') { processor.clear(); self.postMessage({ type: 'TOTAL_LINES', data: 0 }); }
                else if (type === 'SET_LINE_ENDING') processor.set_line_ending(data);
            } catch (e) { console.error(e); }
        };
    } catch (e) { console.error("Worker Init Failed", e); }
})();
