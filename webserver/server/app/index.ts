import type { NextFunction } from 'express';
import express from 'express';
import type { Response as ExResponse, Request as ExRequest } from 'express';
import swaggerUi from 'swagger-ui-express';
import bodyParser from 'body-parser';
import { RegisterRoutes } from '../build/routes';
import SwaggerSingleton from './models/SwaggerSingleton';
import { ValidateError } from 'tsoa';
import { StatusCodes } from 'http-status-codes';
import { Errors, genErrorMessage } from '../../shared/errors';

export const app = express();

app.use(
  bodyParser.urlencoded({
    extended: true,
  })
);
app.use(bodyParser.json());

RegisterRoutes(app);

app.use('/docs', swaggerUi.serve, async (_req: ExRequest, res: ExResponse) => {
  return res.send(swaggerUi.generateHTML(await SwaggerSingleton()));
});

app.use(function notFoundHandler(_req, res: ExResponse) {
  res.status(404).send({
    message: 'Not Found',
  });
});

app.use(function errorHandler(
  err: unknown,
  req: ExRequest,
  res: ExResponse,
  next: NextFunction
): ExResponse | void {
  if (err instanceof ValidateError) {
    return res
      .status(StatusCodes.UNPROCESSABLE_ENTITY)
      .json(genErrorMessage(Errors.IncorrectFormat, err?.fields));
  }
  if (err instanceof Error) {
    return res.status(500).json({
      message: 'Internal Server Error',
    });
  }

  next();
});
