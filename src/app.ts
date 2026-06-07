import "reflect-metadata";
import express, { Request, Response, NextFunction } from "express";
import path from 'path';
import "express-async-errors";
import createConnection from "./database";
import cors from 'cors';

import { router } from "./routes";
import { AppError } from "./errors/AppError";
import errorHandler from './errors/Handler';

createConnection();

const app = express();

app.use(cors());
app.use(express.json());
app.use(router);
app.use('/uploads', express.static(path.join(__dirname,'..','uploads')));

app.use(errorHandler);
  
  export { app };