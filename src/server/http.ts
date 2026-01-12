import express, { Express, Request, Response, NextFunction } from 'express';
import { registerApiRoutes } from '../api';
import { logger } from '../utils/logger';

export function createServer(): Express {
    const app = express();

    // Middleware
    app.use(express.json({ limit: '1000mb' }));

    // Health check
    app.get('/health', (_req: Request, res: Response) => {
        res.json({ status: 'ok' });
    });

    // Register API routes
    registerApiRoutes(app);

    // Error handling middleware
    app.use(
        (
            err: unknown,
            _req: Request,
            res: Response,
            _next: NextFunction
        ) => {
            logger.error('Unhandled error', err);
            res.status(500).json({
                error: {
                    message: 'Internal server error'
                }
            });
        }
    );

    return app;
}

export function startHttpServer(port: number): void {
    const app = createServer();

    const server = app.listen(port, () => {
        logger.info(`Local LLM server listening on http://localhost:${port}`);
    });

    server.on('error', (error) => {
        logger.error('HTTP server error', error);
        process.exit(1);
    });
}
