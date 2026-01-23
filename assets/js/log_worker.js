// Web Worker for OPFS Log Storage - Rust Refactored version (Full Parity)
import init, { LogProcessor } from "/wasm/serial_monitor.js";

let processor;
let isInitialized = false;
let searchSessionId = 0;
let isProcessing = false; // Re-entrancy guard

// Throttling State
let lastNotifyTime = 0;
let notifyTimer = null;
const NOTIFY_INTERVAL = 50;

function scheduleUpdate(count) {
    if (typeof count !== 'number') return;
    const now = Date.now();
    if (now - lastNotifyTime > NOTIFY_INTERVAL) {
        self.postMessage({ type: 'TOTAL_LINES', data: count });
        lastNotifyTime = now;
        if (notifyTimer) {
            clearTimeout(notifyTimer);
            notifyTimer = null;
        }
    } else {
        if (!notifyTimer) {
            notifyTimer = setTimeout(() => {
                self.postMessage({ type: 'TOTAL_LINES', data: count });
                lastNotifyTime = Date.now();
                notifyTimer = null;
            }, NOTIFY_INTERVAL);
        }
    }
}

async function start() {
    console.log("[LogWorker] Start function begins...");
    try {
        await init();
        console.log("[LogWorker] WASM Init successful.");

        processor = new LogProcessor();
        console.log("[LogWorker] LogProcessor instance created.");

        const root = await navigator.storage.getDirectory();
        const fileName = `session_logs_${Date.now()}.txt`;
        const fileHandle = await root.getFileHandle(fileName, { create: true });
        const syncAccessHandle = await fileHandle.createSyncAccessHandle();

        console.log("[LogWorker] SyncAccessHandle created successfully.");

        processor.set_sync_handle(syncAccessHandle);
        isInitialized = true;

        console.log(`[LogWorker] JS-Rust Bridge Active. Engine Initialized.`);
        self.postMessage({ type: 'INITIALIZED', data: fileName });
    } catch (e) {
        console.error("[LogWorker] Critical Init Error:", e);
        self.postMessage({ type: 'ERROR', data: "Failed to initialize Rust LogProcessor: " + e.message });
    }
}

start();

// Use synchronous onmessage to ensure messages are processed strictly in order
// and to avoid concurrent calls into the WASM instance if the event loop nests.
self.onmessage = (e) => {
    if (!isInitialized) return;
    if (isProcessing) {
        // This shouldn't happen in a worker unless something synchronous yields
        console.warn("[LogWorker] Re-entrant message detected, queuing or dropping?", e.data.type);
    }

    isProcessing = true;
    const msg = e.data;
    const type = msg.type;
    const data = msg.data;

    try {
        switch (type) {
            case 'SET_LINE_ENDING':
                processor.set_line_ending(data);
                break;

            case 'APPEND_CHUNK':
                processor.append_chunk(data.chunk, data.is_hex);
                scheduleUpdate(processor.get_line_count());
                break;

            case 'REQUEST_WINDOW': {
                const lines = processor.request_window(data.startLine, data.count);
                self.postMessage({
                    type: 'LOG_WINDOW',
                    data: { startLine: data.startLine, lines }
                });
                break;
            }

            case 'SEARCH_LOGS':
                searchSessionId++;
                const currentSession = searchSessionId;
                processor.search_logs(
                    data.query,
                    data.match_case,
                    data.use_regex,
                    data.invert
                );
                if (currentSession === searchSessionId) {
                    scheduleUpdate(processor.get_line_count());
                }
                break;

            case 'CLEAR':
                processor.clear();
                self.postMessage({ type: 'TOTAL_LINES', data: 0 });
                break;

            case 'EXPORT_LOGS': {
                const includeTimestamp = !(data && data.include_timestamp === false);
                try {
                    const stream = processor.export_logs(includeTimestamp);
                    self.postMessage({ type: 'EXPORT_STREAM', stream }, [stream]);
                } catch (err) {
                    console.error("[LogWorker] Export Error:", err);
                }
                break;
            }
        }
    } catch (err) {
        console.error(`[LogWorker] Runtime Error in ${type}:`, err);
    } finally {
        isProcessing = false;
    }
};
