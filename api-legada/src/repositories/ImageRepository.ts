import { Entity, EntityRepository, Repository } from "typeorm";

import Images from "../models/Image";

@EntityRepository(Images)
class ImageRepository extends Repository<Images> {}

export { ImageRepository };
