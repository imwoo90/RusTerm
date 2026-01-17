// Web Worker for OPFS Log Storage (Robust version)
let fileHandle;
let syncAccessHandle;
let lineOffsets = [0];
let lineCount = 0;

async function initOPFS() {
    try {
        const root = await navigator.storage.getDirectory();

        // 고유 파일명 대신 고정 파일명을 쓰되, 이미 열려있으면 닫으려고 시도하거나 다른 이름을 씁니다.
        // 여기서는 안전하게 매번 새로운 이름을 쓰겠습니다. (임시 로그 성격)
        const fileName = `session_logs_${Date.now()}.txt`;
        fileHandle = await root.getFileHandle(fileName, { create: true });
        syncAccessHandle = await fileHandle.createSyncAccessHandle();

        console.log(`[LogWorker] Initialized OPFS: ${fileName}`);

        // 초기화 완료 메시지 (선택 사항)
        self.postMessage({ type: 'INITIALIZED', data: fileName });

    } catch (e) {
        console.error("[LogWorker] Init Error:", e);
    }
}

initOPFS();

self.onmessage = async (e) => {
    // 전달받은 데이터가 객체인지 확인
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

            // 너무 자주 보내지 않기 위해 조절할 수도 있지만, 현재는 매번 보냄
            self.postMessage({ type: 'TOTAL_LINES', data: lineCount });
        } catch (err) {
            console.error("[LogWorker] Write Error:", err);
        }
    }

    if (type === 'REQUEST_WINDOW') {
        const { startLine, count } = data;
        if (!syncAccessHandle) return;

        // 경계값 처리
        const start = Math.max(0, Math.min(startLine, lineCount));
        const end = Math.min(start + count, lineCount);
        const effectiveCount = end - start;

        if (effectiveCount <= 0) {
            self.postMessage({ type: 'LOG_WINDOW', data: { startLine: start, lines: [] } });
            return;
        }

        try {
            const startOffset = lineOffsets[start];
            const endOffset = lineOffsets[end];
            const size = endOffset - startOffset;

            const readBuffer = new Uint8Array(size);
            const bytesRead = syncAccessHandle.read(readBuffer, { at: startOffset });

            const decoder = new TextDecoder();
            const text = decoder.decode(readBuffer.slice(0, bytesRead));

            // 마지막 개행 제거 후 분리
            const lines = text.endsWith('\n') ? text.slice(0, -1).split('\n') : text.split('\n');

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
};
