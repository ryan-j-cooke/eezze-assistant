import { ServerResponse } from 'http';

/**
 * Initialize an SSE stream response.
 */
export function startStream(res: ServerResponse): void {
    res.writeHead(200, {
        'Content-Type': 'text/event-stream',
        'Cache-Control': 'no-cache',
        'Connection': 'keep-alive'
    });

    res.write('\n');
}

/**
 * Write a single SSE data event.
 */
export function streamEvent(res: ServerResponse, data: unknown): void {
    res.write(`data: ${JSON.stringify(data)}\n\n`);
}

/**
 * Close an SSE stream.
 */
export function endStream(res: ServerResponse): void {
    res.write('data: [DONE]\n\n');
    res.end();
}
