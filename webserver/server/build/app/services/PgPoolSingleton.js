"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const pg_1 = require("pg");
const submit = pg_1.Query.prototype.submit;
pg_1.Query.prototype.submit = function () {
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
const pool = new pg_1.Pool({
    connectionString: process.env.DATABASE_URL,
    log: console.log,
});
pool.query = pool.query;
exports.default = pool;
