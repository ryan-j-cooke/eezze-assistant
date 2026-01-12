/**
 * High-resolution timer utility.
 */
export class Timer {
    private startTime: number;

    constructor() {
        this.startTime = Date.now();
    }

    /**
     * Reset the timer start point.
     */
    reset(): void {
        this.startTime = Date.now();
    }

    /**
     * Elapsed time in milliseconds.
     */
    elapsed(): number {
        return Date.now() - this.startTime;
    }
}

/**
 * Measure execution time of an async function.
 */
export async function measure<T>(
    fn: () => Promise<T>
): Promise<{ result: T; durationMs: number }> {
    const timer = new Timer();
    const result = await fn();

    return {
        result,
        durationMs: timer.elapsed()
    };
}
