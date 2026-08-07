import { Router } from "express";
import multer from 'multer';

import { CollectiveController } from "./controllers/CollectiveController";
import uploadConfig from './config/upload';

const router = Router();
const upload = multer(uploadConfig);

const collectiveController = new CollectiveController();

router.post("/collectives", upload.array('images'),collectiveController.create);
router.get("/collectives/:id", collectiveController.show);
router.get("/collectives", collectiveController.index);

export { router };