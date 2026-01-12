import { Express } from 'express';
import { EXPERT_RECURSIVE_LOCAL } from '../../eezze.config';

/**
 * Registers OpenAI-compatible models routes
 */
export function registerModelRoutes(app: Express): void {
	app.get("/v1/models", (_request, response) => {
		return response.json({
			object: "list",
			data: [
				{
					id: EXPERT_RECURSIVE_LOCAL,
					object: "model",
					created: 0,
					owned_by: "local",
				},
			],
		});
	});
}
