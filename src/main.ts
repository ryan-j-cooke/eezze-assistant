import { createServer } from './server/http';
import { loadConfig, RuntimeConfig } from './types/config';
import { logger } from './utils/logger';
import { EXPERT_EMBEDDING_DEFAULT, EXPERT_RECURSIVE_LOCAL } from '../eezze.config';

async function checkDependancies(config: RuntimeConfig) {
	const requiredModels = new Set<string>();

	for (const model of Object.values(config.models)) {
		if (model.provider === 'ollama') {
			requiredModels.add(model.name);
		}
	}

	requiredModels.add(EXPERT_EMBEDDING_DEFAULT);
	requiredModels.add(EXPERT_RECURSIVE_LOCAL);

	const requiredModelList = Array.from(requiredModels);

	logger.info('startup.checkDependencies.start', {
		requiredModels: requiredModelList,
	});

	const missing: string[] = [];

	for (const modelName of requiredModelList) {
		try {
			const response = await fetch('http://localhost:11434/api/show', {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json',
				},
				body: JSON.stringify({ name: modelName }),
			});

			if (!response.ok) {
				missing.push(modelName);
				logger.error('startup.modelCheck.httpError', {
					model: modelName,
					status: response.status,
				});
			}
			else {
				logger.info('startup.modelCheck.ok', {
					model: modelName,
				});
			}
		}
		catch (error: any) {
			missing.push(modelName);
			logger.error('startup.modelCheck.failed', {
				model: modelName,
				error: error?.message ?? String(error),
			});
		}
	}

	if (missing.length > 0) {
		logger.error('startup.modelsMissing', {
			missingModels: missing,
		});
		throw new Error(`Required Ollama models missing: ${missing.join(', ')}`);
	}

	logger.info('startup.checkDependencies.success', {
		requiredModels: requiredModelList,
	});
}

async function bootstrap() {
	try {
		const config = loadConfig();
		// await checkDependancies(config);

		const app = createServer();

		const server = app.listen(config.server.port, '0.0.0.0', () => {
			logger.info(`🚀 Recursive LLM server running at http://0.0.0.0:${config.server.port}`);
		});

		// Graceful shutdown
		const shutdown = async (signal: string) => {
			logger.info(`🛑 Received ${signal}, shutting down...`);
			try {
				server.close();
				process.exit(0);
			}
			catch (err) {
				logger.error('Error during shutdown', err);
				process.exit(1);
			}
		};

		process.on('SIGINT', shutdown);
		process.on('SIGTERM', shutdown);
	}
	catch (err) {
		logger.error('❌ Failed to start server', err);
		process.exit(1);
	}
}

bootstrap();
