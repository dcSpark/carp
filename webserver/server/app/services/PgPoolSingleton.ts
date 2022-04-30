import { Query, Pool } from 'pg';

if (process.env.NODE_ENV === 'development') {
  const submit = Query.prototype.submit;
  Query.prototype.submit = function () {
    // @ts-ignore
    const text = this.text;
    // @ts-ignore
    const values = this.values;
    if (values == null) {
      // @ts-ignore
      submit.apply(this, arguments);
      return;
    }
    // @ts-ignore
    const query = values.reduce((q, v, i) => q.replace(`$${i + 1}`, v), text);
    console.log(query);
    // @ts-ignore
    submit.apply(this, arguments);
  };
}

const pool = new Pool({
  connectionString: process.env.DATABASE_URL,
  log: console.log,
});

export default pool;
