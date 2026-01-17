// Web Worker for OPFS Log Storage (Robust version)
let fileHandle;
let syncAccessHandle;
let lineOffsets = [0];
let lineCount = 0;

// Filter State
let isFiltering = false;
let filteredLines = []; // [{start, end}, ...]
let searchSessionId = 0;
// Store active filter options for realtime filtering
let activeFilter = null; // { query, regex, match_case, invert }

// Throttling State
let lastNotifyTime = 0;
let notifyTimer = null;
const NOTIFY_INTERVAL = 50; // Update UI at most every 50ms (20fps)

function scheduleUpdate() {
    const now = Date.now();
    if (now - lastNotifyTime > NOTIFY_INTERVAL) {
        // Send immediately
        postTotalLines();
        lastNotifyTime = now;
        if (notifyTimer) {
            clearTimeout(notifyTimer);
            notifyTimer = null;
        }
    } else {
        // Schedule deferred update (trailing edge)
        if (!notifyTimer) {
            notifyTimer = setTimeout(() => {
                postTotalLines();
                lastNotifyTime = Date.now();
                notifyTimer = null;
            }, NOTIFY_INTERVAL);
        }
    }
}

function postTotalLines() {
    const count = (isFiltering && activeFilter) ? filteredLines.length : lineCount;
    self.postMessage({ type: 'TOTAL_LINES', data: count });
}

async function initOPFS() {
    try {
        const root = await navigator.storage.getDirectory();

        // Cleanup: remove old session files
        for await (const name of root.keys()) {
            if (name.startsWith('session_logs_')) {
                try {
                    await root.removeEntry(name);
                } catch (e) { }
            }
        }

        const fileName = `session_logs_${Date.now()}.txt`;
        fileHandle = await root.getFileHandle(fileName, { create: true });
        syncAccessHandle = await fileHandle.createSyncAccessHandle();

        console.log(`[LogWorker] Initialized OPFS: ${fileName}`);
        self.postMessage({ type: 'INITIALIZED', data: fileName });

    } catch (e) {
        console.error("[LogWorker] Init Error:", e);
        self.postMessage({ type: 'ERROR', data: "Failed to initialize OPFS storage." });
    }
}

initOPFS();

self.onmessage = async (e) => {
    const msg = e.data;
    const type = msg.type;
    const data = msg.data;

    if (type === 'APPEND_LOG') {
        if (!syncAccessHandle) return;

        const text = data + '\n';
        const encoder = new TextEncoder();
        const buffer = encoder.encode(text);

        try {
            const pos = syncAccessHandle.getSize();
            syncAccessHandle.write(buffer, { at: pos });

            const newStart = pos;
            const newEnd = pos + buffer.byteLength; // Includes newline

            // Update main index
            lineOffsets.push(newEnd);
            lineCount++;

            // Handle Realtime Filtering
            if (isFiltering && activeFilter) {
                // Check if this new line matches current filter
                let matched = false;
                // Remove trailing newline for matching check
                const lineContent = data;

                if (activeFilter.regex) {
                    try { matched = activeFilter.regex.test(lineContent); } catch (e) { }
                } else {
                    matched = activeFilter.match_case
                        ? lineContent.includes(activeFilter.query)
                        : lineContent.toLowerCase().includes(activeFilter.lowerQuery);
                }

                if (activeFilter.invert) matched = !matched;

                if (matched) {
                    filteredLines.push({
                        start: newStart,
                        end: newEnd // Points to next line start
                    });
                    // Force update for filter view
                    scheduleUpdate();
                }
            } else {
                // Normal update
                scheduleUpdate();
            }

        } catch (err) {
            console.error("[LogWorker] Write Error:", err);
        }
    }

    if (type === 'REQUEST_WINDOW') {
        const { startLine, count } = data;
        if (!syncAccessHandle) return;

        const total = isFiltering ? filteredLines.length : lineCount;
        const start = Math.max(0, Math.min(startLine, total));
        const end = Math.min(start + count, total);
        const effectiveCount = end - start;

        if (effectiveCount <= 0) {
            self.postMessage({ type: 'LOG_WINDOW', data: { startLine: start, lines: [] } });
            return;
        }

        try {
            const lines = [];
            const decoder = new TextDecoder();

            if (isFiltering) {
                for (let i = start; i < end; i++) {
                    const meta = filteredLines[i];
                    const size = meta.end - meta.start;
                    const buf = new Uint8Array(size);
                    syncAccessHandle.read(buf, { at: meta.start });
                    const text = decoder.decode(buf);
                    lines.push(text.endsWith('\n') ? text.slice(0, -1) : text);
                }
            } else {
                const startOffset = lineOffsets[start];
                const endOffset = lineOffsets[end];
                const size = endOffset - startOffset;

                const readBuffer = new Uint8Array(size);
                const bytesRead = syncAccessHandle.read(readBuffer, { at: startOffset });
                const text = decoder.decode(readBuffer.slice(0, bytesRead));

                const split = text.endsWith('\n') ? text.slice(0, -1).split('\n') : text.split('\n');
                lines.push(...split);
            }

            self.postMessage({ type: 'LOG_WINDOW', data: { startLine: start, lines } });
        } catch (err) {
            console.error("[LogWorker] Read Error:", err);
        }
    }

    if (type === 'EXPORT_LOGS') {
        // ... (Export logic remains same, omitted for brevity but should be kept if overwriting)
        // Since I'm overwriting, I MUST include the full code.
        const includeTimestamp = data && data.include_timestamp;
        if (!syncAccessHandle) return;

        try {
            syncAccessHandle.flush();
            const fileSize = syncAccessHandle.getSize();
            const sourceStream = new ReadableStream({
                start(controller) { this.offset = 0; },
                pull(controller) {
                    const chunkSize = 64 * 1024;
                    if (this.offset >= fileSize) { controller.close(); return; }
                    const buffer = new Uint8Array(chunkSize);
                    const readBytes = syncAccessHandle.read(buffer, { at: this.offset });
                    if (readBytes === 0) { controller.close(); return; }
                    controller.enqueue(buffer.slice(0, readBytes));
                    this.offset += readBytes;
                }
            });

            let finalStream = sourceStream;
            if (includeTimestamp === false) {
                const textDecoder = new TextDecoderStream();
                const textEncoder = new TextEncoderStream();
                const transformer = new TransformStream({
                    start() { this.buffer = ""; },
                    transform(chunk, controller) {
                        this.buffer += chunk;
                        const lines = this.buffer.split('\n');
                        this.buffer = lines.pop();
                        for (const line of lines) {
                            const clean = line.replace(/^\[\d{2}:\d{2}:\d{2}\.\d{3}\] /, '');
                            controller.enqueue(clean + '\n');
                        }
                    },
                    flush(controller) {
                        if (this.buffer) {
                            const clean = this.buffer.replace(/^\[\d{2}:\d{2}:\d{2}\.\d{3}\] /, '');
                            controller.enqueue(clean);
                        }
                    }
                });
                finalStream = sourceStream.pipeThrough(textDecoder).pipeThrough(transformer).pipeThrough(textEncoder);
            }
            self.postMessage({ type: 'EXPORT_STREAM', stream: finalStream }, [finalStream]);
        } catch (err) {
            console.error("[LogWorker] Stream Export Error:", err);
        }
    }

    if (type === 'CLEAR') {
        if (!syncAccessHandle) return;
        try {
            syncAccessHandle.truncate(0);
            syncAccessHandle.flush();
            lineCount = 0;
            lineOffsets = [0];

            isFiltering = false;
            activeFilter = null;
            filteredLines = [];
            searchSessionId++;

            self.postMessage({ type: 'TOTAL_LINES', data: 0 });
            console.log("[LogWorker] Storage Cleared");
        } catch (err) {
            console.error("[LogWorker] Clear Error:", err);
        }
    }

    if (type === 'SEARCH_LOGS') {
        const { query, match_case, use_regex, invert } = data;

        searchSessionId++;
        const currentSession = searchSessionId;

        if (!query || query.trim() === '') {
            isFiltering = false;
            activeFilter = null;
            filteredLines = [];
            self.postMessage({ type: 'TOTAL_LINES', data: lineCount });
            return;
        }

        // Setup Filter
        isFiltering = true;
        filteredLines = [];

        // Prepare shared filter object
        let regex = null;
        let lowerQuery = "";
        if (use_regex) {
            try { regex = new RegExp(query, match_case ? '' : 'i'); } catch (e) { }
        } else {
            lowerQuery = match_case ? query : query.toLowerCase();
        }

        activeFilter = {
            query,
            lowerQuery,
            match_case,
            regex,
            invert
        };

        try {
            const CHUNK_SIZE = 5000;
            const YIELD_INTERVAL = 100;
            let lastYield = Date.now();

            for (let i = 0; i < lineCount; i++) {
                if (currentSession !== searchSessionId) return;

                const batchEnd = Math.min(i + CHUNK_SIZE, lineCount);
                const batchStartOffset = lineOffsets[i];
                const batchEndOffset = lineOffsets[batchEnd];
                const size = batchEndOffset - batchStartOffset;

                const buf = new Uint8Array(size);
                syncAccessHandle.read(buf, { at: batchStartOffset });
                const batchText = new TextDecoder().decode(buf);
                const batchLines = batchText.endsWith('\n') ? batchText.slice(0, -1).split('\n') : batchText.split('\n');

                for (let j = 0; j < batchLines.length; j++) {
                    const line = batchLines[j];
                    let matched = false;

                    if (activeFilter.regex) {
                        try { matched = activeFilter.regex.test(line); } catch (e) { }
                    } else {
                        matched = activeFilter.match_case
                            ? line.includes(activeFilter.query)
                            : line.toLowerCase().includes(activeFilter.lowerQuery);
                    }

                    if (activeFilter.invert) matched = !matched;

                    if (matched) {
                        const globIdx = i + j;
                        filteredLines.push({
                            start: lineOffsets[globIdx],
                            end: lineOffsets[globIdx + 1]
                        });
                    }
                }

                i = batchEnd - 1;

                if (Date.now() - lastYield > YIELD_INTERVAL) {
                    self.postMessage({ type: 'TOTAL_LINES', data: filteredLines.length });
                    await new Promise(r => setTimeout(r, 0));
                    lastYield = Date.now();
                }
            }

            self.postMessage({ type: 'TOTAL_LINES', data: filteredLines.length });

        } catch (err) {
            console.error(err);
        }
    }
};
