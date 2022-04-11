import { Query, Pool } from 'pg';

const submit = Query.prototype.submit;
Query.prototype.submit = function () {
  // @ts-ignore
  const text = this.text;
  // @ts-ignore
  const values = this.values;
  // @ts-ignore
  const query = values.reduce((q, v, i) => q.replace(`$${i + 1}`, v), text);
  console.log(query);
  // @ts-ignore
  submit.apply(this, arguments);
};

const pool = new Pool({
  connectionString: process.env.DATABASE_URL,
  log: console.log,
});
pool.query = pool.query;

export default pool;
