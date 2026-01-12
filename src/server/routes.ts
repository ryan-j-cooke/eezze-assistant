import { IncomingMessage, ServerResponse } from 'http';
import { handleChatCompletion } from '../api/chat';
import { handleEmbeddings } from '../index/embed';
import { logger } from '../utils/logger';

export async function handleRequest(
    req: IncomingMessage,
    res: ServerResponse
): Promise<void> {
    const method = req.method ?? 'GET';
    const url = req.url ?? '/';

    if (method === 'GET' && url === '/health') {
        return respondJson(res, 200, {
            status: 'ok'
        });
    }

    if (method === 'POST' && url === '/v1/chat/completions') {
        const body = await readJsonBody(req);
        const result = await handleChatCompletion(body);

        console.log('Chat completion result:', result);

        return respondJson(res, 200, result);
    }

    if (method === 'POST' && url === '/v1/embeddings') {
        const body = await readJsonBody(req);
        const result = await handleEmbeddings(body);

        console.log('Embeddings result:', result);

        return respondJson(res, 200, result);
    }

    logger.warn(`Unhandled route: ${method} ${url}`);

    respondJson(res, 404, {
        error: {
            message: 'Not found'
        }
    });
}

function respondJson(
    res: ServerResponse,
    status: number,
    payload: unknown
): void {
    res.statusCode = status;
    res.setHeader('Content-Type', 'application/json');
    res.end(JSON.stringify(payload));
}

function readJsonBody(req: IncomingMessage): Promise<any> {
    return new Promise((resolve, reject) => {
        let data = '';

        req.on('data', (chunk) => {
            data += chunk;
        });

        req.on('end', () => {
            try {
                resolve(JSON.parse(data || '{}'));
            } catch (error) {
                reject(error);
            }
        });

        req.on('error', reject);
    });
}
