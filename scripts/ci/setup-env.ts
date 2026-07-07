import { writeFileSync, readFileSync } from 'node:fs';

const ENV_EXAMPLE_PATH = '.env.example';
const ENV_PATH = '.env';

/**
 * CI environment overrides for the .env file.
 * These values ensure the application runs correctly within the GitHub Actions Docker network.
 */
const overrides: Record<string, string> = {
  DATABASE_URL: 'postgres://utopia:utopia@postgres:5432/utopia?sslmode=disable',
  APP_STRICT_SSL: 'false',
  BOOTSTRAP_KEY: process.env.BOOTSTRAP_KEY || 'ci-test-bootstrap-key-2026',
};

function setupEnv(): void {
  try {
    const content = readFileSync(ENV_EXAMPLE_PATH, 'utf8');
    const lines = content.split('\n');

    const result = lines
      .map((line) => {
        // Handle comments or empty lines
        if (!line || line.trim().startsWith('#')) {
          return line;
        }

        const [key] = line.split('=');
        const trimmedKey = key.trim();

        if (trimmedKey && overrides[trimmedKey]) {
          return `${trimmedKey}=${overrides[trimmedKey]}`;
        }
        return line;
      })
      .join('\n');

    writeFileSync(ENV_PATH, result);
    console.log('✅ .env file generated for CI successfully');
  } catch (error) {
    console.error('❌ Failed to generate .env file:', error);
    process.exit(1);
  }
}

setupEnv();
