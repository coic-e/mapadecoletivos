import { Entity, EntityRepository, Repository } from "typeorm";

import Collectives from "../models/Collectives";

@EntityRepository(Collectives)
class CollectiveRepository extends Repository<Collectives> {}

export { CollectiveRepository };
