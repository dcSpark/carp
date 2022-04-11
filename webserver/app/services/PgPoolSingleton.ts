import { Pool } from 'pg';

export default new Pool({
  connectionString: process.env.DATABASE_URL,
});
