import { Express } from 'express';
import { registerChatRoutes } from './chat';
import { registerModelRoutes } from './models';
import { handleEmbeddings } from '../index/embed';

/**
 * Register all API routes
 */
export function registerApiRoutes(app: Express): void {
	registerModelRoutes(app);
	registerChatRoutes(app);

	app.post('/v1/embeddings', async (request, response) => {
		const body = request.body as any;

		const inputs: string[] = Array.isArray(body?.input)
			? body.input
			: typeof body?.input === 'string'
				? [body.input]
				: [];

		if (inputs.length === 0) {
			return response.status(400).json({
				error: {
					message: 'Invalid request: input is required',
					type: 'invalid_request_error',
				},
			});
		}

		try {
			const embeddings = await Promise.all(
				inputs.map((text) =>
					handleEmbeddings(text, {
						model: body?.model,
					})
				)
			);

			return response.json({
				object: 'list',
				data: embeddings.map((result, index) => ({
					object: 'embedding',
					embedding: result.vector,
					index,
				})),
				model: body?.model ?? embeddings[0].model,
			});
		}
		catch (error: any) {
			return response.status(500).json({
				error: {
					message: error?.message ?? 'Internal server error',
					type: 'internal_error',
				},
			});
		}
	});
}
