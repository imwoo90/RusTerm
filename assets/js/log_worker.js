// Web Worker for OPFS Log Storage (Robust version)
let fileHandle;
let syncAccessHandle;
let lineOffsets = [0];
let lineCount = 0;

// Filter State
let isFiltering = false;
let filteredLines = []; // [{start, end}, ...]
let searchSessionId = 0;

async function initOPFS() {
    try {
        const root = await navigator.storage.getDirectory();

        // Cleanup: remove old session files to prevent storage bloat
        for await (const name of root.keys()) {
            if (name.startsWith('session_logs_')) {
                try {
                    await root.removeEntry(name);
                } catch (e) {
                    // Ignore if file is in use by another tab
                }
            }
        }

        // Use a unique name for the current session
        const fileName = `session_logs_${Date.now()}.txt`;
        fileHandle = await root.getFileHandle(fileName, { create: true });
        syncAccessHandle = await fileHandle.createSyncAccessHandle();

        console.log(`[LogWorker] Initialized OPFS: ${fileName}`);

        // Initialization complete message (optional)
        self.postMessage({ type: 'INITIALIZED', data: fileName });

    } catch (e) {
        console.error("[LogWorker] Init Error:", e);
        // Notify main thread about OPFS failure
        self.postMessage({ type: 'ERROR', data: "Failed to initialize OPFS storage. Your browser may not support it or storage is full." });
    }
}

initOPFS();

self.onmessage = async (e) => {
    // Check if received data is an object
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

            lineCount++;
            lineOffsets.push(syncAccessHandle.getSize());

            // If filtering is active, we should check if new line matches query separately
            // But for simplicity, we might just notify TOTAL_LINES if not filtering.
            // If filtering, we might need to "Append to Filter" (TODO optimization)
            if (!isFiltering) {
                // Can be throttled to avoid sending too frequently, but currently sent every time
                self.postMessage({ type: 'TOTAL_LINES', data: lineCount });
            }
        } catch (err) {
            console.error("[LogWorker] Write Error:", err);
        }
    }

    if (type === 'REQUEST_WINDOW') {
        const { startLine, count } = data;
        if (!syncAccessHandle) return;

        // Use filtered count if filtering
        const total = isFiltering ? filteredLines.length : lineCount;

        // Handle boundary values
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
                // Read Scattered Lines
                for (let i = start; i < end; i++) {
                    const meta = filteredLines[i];
                    const size = meta.end - meta.start;
                    const buf = new Uint8Array(size);
                    syncAccessHandle.read(buf, { at: meta.start });
                    const text = decoder.decode(buf);
                    // Remove trailing newline if present
                    lines.push(text.endsWith('\n') ? text.slice(0, -1) : text);
                }
            } else {
                // Read Contiguous Block logic (Old Logic)
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
        const includeTimestamp = data && data.include_timestamp;
        if (!syncAccessHandle) return;

        try {
            syncAccessHandle.flush();
            const fileSize = syncAccessHandle.getSize();

            // 1. Create Source Stream from OPFS
            const sourceStream = new ReadableStream({
                start(controller) {
                    this.offset = 0;
                },
                pull(controller) {
                    const chunkSize = 64 * 1024; // 64KB
                    if (this.offset >= fileSize) {
                        controller.close();
                        return;
                    }

                    const buffer = new Uint8Array(chunkSize);
                    // syncAccessHandle.read is synchronous
                    const readBytes = syncAccessHandle.read(buffer, { at: this.offset });

                    if (readBytes === 0) {
                        controller.close();
                        return;
                    }

                    // Slice if read less than chunk size
                    controller.enqueue(buffer.slice(0, readBytes));
                    this.offset += readBytes;
                }
            });

            let finalStream = sourceStream;

            // 2. Apply Timestamp Filter if needed
            if (includeTimestamp === false) {
                const textDecoder = new TextDecoderStream();
                const textEncoder = new TextEncoderStream();

                const transformer = new TransformStream({
                    start() { this.buffer = ""; },
                    transform(chunk, controller) {
                        this.buffer += chunk;
                        const lines = this.buffer.split('\n');
                        this.buffer = lines.pop(); // Keep incomplete line

                        for (const line of lines) {
                            // Remove timestamp [HH:MM:SS.ms] (15 chars) + space
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

            // 3. Send Stream to Main Thread
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
            // Clear filter state
            isFiltering = false;
            filteredLineOffsets = [0];
            filteredLineCount = 0;
            searchSessionId++;

            self.postMessage({ type: 'TOTAL_LINES', data: 0 });
            console.log("[LogWorker] Storage Cleared");
        } catch (err) {
            console.error("[LogWorker] Clear Error:", err);
        }
    }

    if (type === 'SEARCH_LOGS') {
        const { query, match_case, use_regex, invert } = data;

        // Start new search session (invalidates old ones)
        searchSessionId++;
        const currentSession = searchSessionId;

        // Reset if query is empty
        if (!query || query.trim() === '') {
            isFiltering = false;
            filteredLines = [];
            // Restore total lines to original count
            self.postMessage({ type: 'TOTAL_LINES', data: lineCount });
            return;
        }

        // Prepare Search
        isFiltering = true;
        filteredLines = []; // Array of {start, end}

        try {
            let regex = null;
            let lowerQuery = "";
            if (use_regex) {
                regex = new RegExp(query, match_case ? '' : 'i');
            } else {
                lowerQuery = match_case ? query : query.toLowerCase();
            }

            // Progressive Forward Scan with Yield
            const CHUNK_SIZE = 5000;
            const YIELD_INTERVAL = 100; // ms
            let lastYield = Date.now();

            for (let i = 0; i < lineCount; i++) {
                // Check abort
                if (currentSession !== searchSessionId) return;

                // Batched loop
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

                    if (regex) {
                        try { matched = regex.test(line); } catch (e) { }
                    } else {
                        matched = match_case ? line.includes(query) : line.toLowerCase().includes(lowerQuery);
                    }

                    if (invert) matched = !matched;

                    if (matched) {
                        const globIdx = i + j;
                        filteredLines.push({
                            start: lineOffsets[globIdx],
                            end: lineOffsets[globIdx + 1]
                        });
                    }
                }

                i = batchEnd - 1; // Loop increment will do i++

                // Yield and Update UI progressively
                if (Date.now() - lastYield > YIELD_INTERVAL) {
                    self.postMessage({ type: 'TOTAL_LINES', data: filteredLines.length });
                    // Also request refresh of current window?
                    // For now, just update scrollbar. Auto-scroller will handle rest?
                    // Actually main thread needs to know to re-request window.

                    await new Promise(r => setTimeout(r, 0));
                    lastYield = Date.now();
                }
            }

            // Final Update
            self.postMessage({ type: 'TOTAL_LINES', data: filteredLines.length });

        } catch (err) {
            console.error(err);
        }
    }
};
