import init, { LogProcessor } from "/wasm/serial_monitor.js";

let processor;
let lastNotify = 0;
let pending = null;

const notify = (count) => {
    const now = Date.now();
    if (now - lastNotify > 50) {
        lastNotify = now;
        self.postMessage({ type: 'TOTAL_LINES', data: count });
        if (pending) clearTimeout(pending);
        pending = null;
    } else if (!pending) {
        pending = setTimeout(() => notify(processor.get_line_count()), 50);
    }
};

async function start() {
    try {
        await init();
        processor = new LogProcessor();
        const root = await navigator.storage.getDirectory();
        const handle = await (await root.getFileHandle(`logs_${Date.now()}.txt`, { create: true })).createSyncAccessHandle();
        processor.set_sync_handle(handle);
        self.postMessage({ type: 'INITIALIZED' });
    } catch (e) {
        console.error("Worker init failed:", e);
    }
}

start();

self.onmessage = (e) => {
    const { type, data } = e.data;
    try {
        switch (type) {
            case 'APPEND_CHUNK':
                processor.append_chunk(data.chunk, data.is_hex);
                notify(processor.get_line_count());
                break;
            case 'REQUEST_WINDOW':
                self.postMessage({
                    type: 'LOG_WINDOW', data: {
                        startLine: data.startLine,
                        lines: processor.request_window(data.startLine, data.count)
                    }
                });
                break;
            case 'SEARCH_LOGS':
                processor.search_logs(data.query, data.match_case, data.use_regex, data.invert);
                notify(processor.get_line_count());
                break;
            case 'EXPORT_LOGS':
                const stream = processor.export_logs(!(data?.include_timestamp === false));
                self.postMessage({ type: 'EXPORT_STREAM', stream }, [stream]);
                break;
            case 'CLEAR':
                processor.clear();
                self.postMessage({ type: 'TOTAL_LINES', data: 0 });
                break;
            case 'SET_LINE_ENDING':
                processor.set_line_ending(data);
                break;
        }
    } catch (err) {
        console.error(`[Worker] ${type} error:`, err);
    }
};
