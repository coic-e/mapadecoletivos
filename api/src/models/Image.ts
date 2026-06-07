import { Column, CreateDateColumn, Entity, PrimaryGeneratedColumn, ManyToOne, JoinColumn } from "typeorm";
import Collective from "./Collectives";

@Entity('images')
export default class Image {
    @PrimaryGeneratedColumn('increment')
    id: number;

    @Column()
    path: string;

    @ManyToOne(() => Collective, collective => collective.images)
    @JoinColumn({name: 'collective_id' })
    collective: Collective;
}