import { app } from './index';

const port = process.env.PORT ?? 3000;

// eslint-disable-next-line no-console
app.listen(port, () => console.log(`Carp listening at http://localhost:${port}`));
