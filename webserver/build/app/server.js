"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const index_1 = require("./index");
const port = process.env.PORT || 3000;
index_1.app.listen(port, () => console.log(`Example app listening at http://localhost:${port}`));
