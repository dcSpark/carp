import { Query, Pool } from 'pg';

/* eslint-disable */

if (process.env.NODE_ENV === 'development') {
  const submit = Query.prototype.submit;
  Query.prototype.submit = function (...args) {
    // @ts-ignore
    const text = this.text;
    // @ts-ignore
    const values = this.values;
    if (values == null) {
      // @ts-ignore
      submit.apply(this, args);
      return;
    }
    // @ts-ignore
    const query = values.reduce((q, v, i) => q.replace(`$${i + 1}`, v), text);
    console.log(query);
    // @ts-ignore
    submit.apply(this, args);
  };
}

/* eslint-enable */

const pool = new Pool({
  connectionString: process.env.DATABASE_URL,
});

export default pool;
