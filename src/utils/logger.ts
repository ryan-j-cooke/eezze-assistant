type LogLevel = 'debug' | 'info' | 'warn' | 'error';

function log(level: LogLevel, message: string, meta?: unknown): void {
    const entry: Record<string, unknown> = {
        timestamp: new Date().toISOString(),
        level,
        message
    };

    if (meta !== undefined) {
        if (meta instanceof Error) {
            entry.meta = {
                name: meta.name,
                message: meta.message,
                stack: meta.stack
            };
        }
        else {
            entry.meta = meta;
        }
    }

    const output = JSON.stringify(entry);

    if (level === 'error') {
        console.error(output);
    }
    else {
        console.log(output);
    }
}

export const logger = {
    debug(message: string, meta?: unknown): void {
        log('debug', message, meta);
    },

    info(message: string, meta?: unknown): void {
        log('info', message, meta);
    },

    warn(message: string, meta?: unknown): void {
        log('warn', message, meta);
    },

    error(message: string, meta?: unknown): void {
        log('error', message, meta);
    }
};
