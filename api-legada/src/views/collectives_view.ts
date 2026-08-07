import Collective from '../models/Collectives'
import imagesView from './images_view'

export default {
    render(collective: Collective){
        return {
            id: collective.id,
            name: collective.name,
            latitude: collective.latitude,
            longitude: collective.longitude,
            type: collective.type,
            city: collective.city,
            uf: collective.uf,
            email: collective.email,
            social: collective.social,
            about: collective.about,
            images: imagesView.renderMany(collective.images)
        };
    },

    renderMany(collectives: Collective[]){
        return collectives.map(collective => this.render(collective));
    }
};