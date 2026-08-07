import { Column, CreateDateColumn, Entity, PrimaryGeneratedColumn, OneToMany, JoinColumn } from "typeorm";
import Image from "./Image"; 

@Entity('collectives')
export default class Collectives {
    @PrimaryGeneratedColumn('increment')
    id: number;

    @Column()
    name: string;

    @Column()
    latitude: number; 
    
    @Column()
    longitude: number;

    @Column()
    type: string;

    @Column()
    city: string;

    @Column()
    uf: string;

    @Column()
    email: string;

    @Column()
    social: string;

    @Column()
    about: string;

    @CreateDateColumn()
    created_at: Date;

    @OneToMany(() => Image, image => image.collective, {
        cascade: ['insert', 'update']
    })
    @JoinColumn({name: 'collective_id'})
    images: Image[];
}