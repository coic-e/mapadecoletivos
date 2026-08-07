import { ErrorRequestHandler ,Request, Response, NextFunction } from "express";
import { AppError } from "./AppError";
import { ValidationError } from "yup"; 

interface ValidationErrors {
  [key: string]: string[];
};

const errorHandler: ErrorRequestHandler = (
  err: Error,
  request: Request,
  response: Response,
  _next: NextFunction
) => {
  if (err instanceof ValidationError) {

    let errors: ValidationErrors = {};

    err.inner.forEach(element => {
      errors[err.path!] = err.errors;
    });

    return response.status(400).json({
      message: "Validation Fails",
      errors
    });
  }
  
  console.log(err);

  return response.status(500).json({
    status: "Error",
    message: `Internal server error ${err.message}`,
  });
};

export default errorHandler;
