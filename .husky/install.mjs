// Husky install script
import { fileURLToPath } from 'node:url';
import { writeFileSync, mkdirSync, existsSync } from 'node:fs';
import { dirname, join } from 'node:path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const huskyDir = join(__dirname, '.husky');

if (!existsSync(huskyDir)) {
  mkdirSync(huskyDir, { recursive: true });
}

// This file ensures the .husky directory exists
// Husky v9+ doesn't require the _/husky.sh file anymore

