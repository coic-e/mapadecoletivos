import { Request, Response } from "express";
import { getCustomRepository } from "typeorm";
import { CollectiveRepository } from "../repositories/CollectiveRepository";
import { AppError } from "../errors/AppError";
import collectiveView from "../views/collectives_view";
import * as Yup from "yup";

class CollectiveController {
  async index(request: Request, response: Response) {
    const collectivesRepository = getCustomRepository(CollectiveRepository);

    const all = await collectivesRepository.find({
      relations: ["images"],
    });

    return response.json(collectiveView.renderMany(all));
  }

  async show(request: Request, response: Response) {
    const { id } = request.params;

    const collectivesRepository = getCustomRepository(CollectiveRepository);

    const coletivo = await collectivesRepository.findOneOrFail(id, {
      relations: ["images"],
    });

    return response.json(collectiveView.render(coletivo));
  }

  async create(request: Request, response: Response) {
    const {
      name,
      latitude,
      longitude,
      type,
      city,
      uf,
      email,
      social,
      about,
    } = request.body;

    const collectivesRepository = getCustomRepository(CollectiveRepository);

    const requestImages = request.files as Express.Multer.File[];
    const images = requestImages.map((image) => {
      return { path: image.filename };
    });

    const data = {
      name,
      latitude,
      longitude,
      type,
      city,
      uf,
      email,
      social,
      about,
      images,
    };

    const schema = Yup.object().shape({
      name: Yup.string().required('Nome é obrigatório'),
      latitude: Yup.number().required('Latitude é obrigatório'),
      longitude: Yup.number().required('Longitude é obrigatório'),
      type: Yup.string().required('Tipo é obrigatório'),
      city: Yup.string().required('Cidade é obrigatório'),
      uf: Yup.string().required('UF é obrigatório'),
      email: Yup.string().email().required(),
      social: Yup.string().required('Social é obrigatório'),
      about: Yup.string().required('Sobre é obrigatório').max(300),
      images: Yup.array(
        Yup.object().shape({
          path: Yup.string().required(),
        })
      ),
    });

    await schema.validate(data, {
      abortEarly: false,
    });

    const coletivo = collectivesRepository.create(data);

    await collectivesRepository.save(coletivo);

    return response.status(201).json(coletivo);
  }
}
export { CollectiveController };
