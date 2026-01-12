import Database from 'better-sqlite3';

export interface StoredEmbedding {
    id: string;
    text: string;
    vector: number[];
    model: string;
}

export interface QueryResult {
    id: string;
    text: string;
    score: number;
}

export class EmbeddingStore {
    private db: Database.Database;

    constructor(path: string = 'embeddings.db') {
        this.db = new Database(path);
        this.initialize();
    }

    private initialize(): void {
        this.db.exec(`
            CREATE TABLE IF NOT EXISTS embeddings (
                id TEXT PRIMARY KEY,
                text TEXT NOT NULL,
                vector TEXT NOT NULL,
                model TEXT NOT NULL
            );
        `);
    }

    /**
     * Insert or update an embedding.
     */
    upsert(embedding: StoredEmbedding): void {
        const stmt = this.db.prepare(`
            INSERT INTO embeddings (id, text, vector, model)
            VALUES (@id, @text, @vector, @model)
            ON CONFLICT(id) DO UPDATE SET
                text = excluded.text,
                vector = excluded.vector,
                model = excluded.model;
        `);

        stmt.run({
            id: embedding.id,
            text: embedding.text,
            vector: JSON.stringify(embedding.vector),
            model: embedding.model,
        });
    }

    /**
     * Retrieve top-K most similar embeddings.
     */
    query(
        queryVector: number[],
        options?: {
            limit?: number;
            minScore?: number;
            model?: string;
        }
    ): QueryResult[] {
        const limit = options?.limit ?? 5;
        const minScore = options?.minScore ?? 0;
        const model = options?.model;

        const rows = model
            ? this.db
                  .prepare(
                      'SELECT id, text, vector FROM embeddings WHERE model = ?'
                  )
                  .all(model)
            : this.db
                  .prepare(
                      'SELECT id, text, vector FROM embeddings'
                  )
                  .all();

        const results: QueryResult[] = [];

        for (const row of rows) {
            const vector = JSON.parse(row.vector) as number[];

            const score = cosineSimilarity(queryVector, vector);

            if (score >= minScore) {
                results.push({
                    id: row.id,
                    text: row.text,
                    score,
                });
            }
        }

        return results
            .sort((a, b) => b.score - a.score)
            .slice(0, limit);
    }

    /**
     * Remove an embedding.
     */
    delete(id: string): void {
        this.db
            .prepare('DELETE FROM embeddings WHERE id = ?')
            .run(id);
    }

    /**
     * Clear the entire store.
     */
    clear(): void {
        this.db.exec('DELETE FROM embeddings');
    }
}

/**
 * Cosine similarity between vectors.
 */
function cosineSimilarity(a: number[], b: number[]): number {
    let dot = 0;
    let normA = 0;
    let normB = 0;

    for (let i = 0; i < a.length; i++) {
        dot += a[i] * b[i];
        normA += a[i] * a[i];
        normB += b[i] * b[i];
    }

    if (normA === 0 || normB === 0) {
        return 0;
    }

    return dot / (Math.sqrt(normA) * Math.sqrt(normB));
}
