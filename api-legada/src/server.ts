import { app } from "./app";
import express, { Request, Response } from "express";
import {getRepository} from 'typeorm';
import Collectives from './models/Collectives';


// Query Params: http://localhost:333/users?search=lucas
// Route Params: http://localhost:333/users/1
// Body: http://localhost:333/users (identificar um recurso)


app.listen(3333, () => console.log("Server is running! PORT", 3333));
