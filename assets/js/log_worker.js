let processor, currentFile = null;
let lastNotifiedCount = 0;
let isInitialized = false;
const messageQueue = [];

// Periodic UI Update (Approx 20Hz / 50ms)
setInterval(() => {
    if (!processor || !isInitialized) return;
    const currentCount = processor.get_line_count();
    if (currentCount !== lastNotifiedCount) {
        lastNotifiedCount = currentCount;
        self.postMessage({ type: 'TOTAL_LINES', data: currentCount });
    }
}, 50);

const getFiles = async (root) => {
    const files = [];
    for await (const [n, h] of root.entries()) if (n.startsWith('logs_') && n.endsWith('.txt')) files.push([n, h]);
    return files.sort((a, b) => parseInt(b[0].split('_')[1]) - parseInt(a[0].split('_')[1]));
};

const getLock = async (fileHandle) => {
    for (let i = 0; i < 20; i++) {
        try { return await fileHandle.createSyncAccessHandle(); }
        catch (e) {
            if (e.name === 'NoModificationAllowedError' || e.name === 'InvalidStateError') {
                await new Promise(r => setTimeout(r, 100));
                continue;
            }
            throw e;
        }
    }
    throw new Error("Failed to acquire OPFS lock after retries");
};

const newSession = async (root, cleanup = false) => {
    if (cleanup && currentFile) try { await root.removeEntry(currentFile); } catch (e) { }
    currentFile = `logs_${Date.now()}.txt`;
    const h = await getLock(await root.getFileHandle(currentFile, { create: true }));
    processor.set_sync_handle(h);
    processor.clear();
    return h;
};

// Start initialization immediately using the URL passed in the worker name
(async () => {
    const wasmUrl = self.name;
    console.log("Worker initializing from name:", wasmUrl);

    if (!wasmUrl || wasmUrl === "undefined") {
        console.error("No WASM URL provided in worker name!");
        return;
    }

    try {
        // Fix for GitHub Pages subpaths:
        // Dioxus glue code might try to fetch WASM from "/./assets/..." which fails on subpaths.
        // We intercept fetch and redirect it to the correct relative path.
        const wasmUrlObj = new URL(wasmUrl, self.location.origin);
        const pathParts = wasmUrlObj.pathname.split('/');
        // Assuming structure: /subpath/assets/script.js or /assets/script.js
        const assetsIdx = pathParts.indexOf('assets');
        const basePath = assetsIdx > 0 ? pathParts.slice(0, assetsIdx).join('/') + '/' : '/';

        console.log("Detected base path for worker:", basePath);

        const originalFetch = self.fetch;
        self.fetch = async (url, options) => {
            let targetUrl = url;
            if (typeof url === 'string' && url.startsWith('/./assets/')) {
                const fixedPath = basePath + url.substring(3); // remove "/./"
                targetUrl = new URL(fixedPath, self.location.origin).toString();
                console.log("Intercepted WASM fetch. Redirecting", url, "to", targetUrl);
            }
            return originalFetch(targetUrl, options);
        };

        const { default: init, LogProcessor } = await import(wasmUrl);
        await init();
        console.log("WASM Initialized in worker");
        processor = new LogProcessor();

        const root = await navigator.storage.getDirectory();
        const files = await getFiles(root);
        if (files.length > 0) {
            currentFile = files[0][0];
            try {
                const h = await getLock(files[0][1]);
                processor.set_sync_handle(h);
                console.log("Resumed session:", currentFile);
            } catch (e) {
                console.warn("Lock failed, starting new session");
                await newSession(root);
            }
            for (let i = 1; i < files.length; i++) try { await root.removeEntry(files[i][0]); } catch (e) { }
        } else {
            await newSession(root);
        }

        isInitialized = true;
        console.log("Worker fully ready. Processing queue:", messageQueue.length);
        self.postMessage({ type: 'INITIALIZED' });
        if (processor.get_line_count() > 0) self.postMessage({ type: 'TOTAL_LINES', data: processor.get_line_count() });

        // Process queued messages
        while (messageQueue.length > 0) {
            const qMsg = messageQueue.shift();
            await handleMessage(qMsg.type, qMsg.data);
        }
    } catch (err) {
        console.error("Worker Async Init Failed:", err);
        self.postMessage({ type: 'ERROR', data: err.message });
    }
})();

// Global message handler
self.onmessage = async (event) => {
    const { type, data } = event.data;

    if (!isInitialized) {
        console.log("Queuing message:", type);
        messageQueue.push({ type, data });
        return;
    }

    await handleMessage(type, data);
};

async function handleMessage(type, data) {
    try {
        const root = await navigator.storage.getDirectory();
        if (type === 'NEW_SESSION') { await newSession(root, true); lastNotifiedCount = 0; self.postMessage({ type: 'TOTAL_LINES', data: 0 }); }
        else if (type === 'APPEND_CHUNK') { processor.append_chunk(data.chunk, data.is_hex); }
        else if (type === 'APPEND_LOG') { processor.append_log(data); }
        else if (type === 'REQUEST_WINDOW') self.postMessage({ type: 'LOG_WINDOW', data: { startLine: data.startLine, lines: processor.request_window(data.startLine, data.count) } });
        else if (type === 'SEARCH_LOGS') { processor.search_logs(data.query, data.match_case, data.use_regex, data.invert); }
        else if (type === 'EXPORT_LOGS') { const s = processor.export_logs(!(data?.include_timestamp === false)); self.postMessage({ type: 'EXPORT_STREAM', stream: s }, [s]); }
        else if (type === 'CLEAR') {
            try { processor.clear(); } catch (err) { console.error("Rust clear failed:", err); }
            lastNotifiedCount = 0;
            self.postMessage({ type: 'TOTAL_LINES', data: 0 });
        }
        else if (type === 'SET_LINE_ENDING') processor.set_line_ending(data);
    } catch (err) {
        console.error("Worker msg processing failed:", err);
    }
}
