import crypto from 'crypto';

/**
 * Generate a stable SHA-256 hash for a given input.
 */
export function hashString(input: string): string {
    return crypto
        .createHash('sha256')
        .update(input)
        .digest('hex');
}

/**
 * Hash multiple strings as a single deterministic key.
 */
export function hashStrings(inputs: string[]): string {
    return hashString(inputs.join('||'));
}
